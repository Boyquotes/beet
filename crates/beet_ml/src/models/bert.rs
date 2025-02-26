//https://github.com/huggingface/candle/blob/main/candle-examples/examples/bert/main.rs
use crate::models::sentence_embeddings::SentenceEmbeddings;
use anyhow::Error as E;
use anyhow::Result;
use bevy::prelude::*;
use candle_core::Tensor;
use candle_nn::VarBuilder;
use candle_transformers::models::bert::BertModel;
use candle_transformers::models::bert::Config;
use candle_transformers::models::bert::HiddenAct;
use candle_transformers::models::bert::DTYPE;
use hf_hub::api::sync::Api;
use hf_hub::Repo;
use hf_hub::RepoType;
use std::borrow::Cow;
use tokenizers::PaddingParams;
use tokenizers::Tokenizer;

#[derive(Clone)]
pub struct BertConfig {
	model_id: Option<String>,
	revision: Option<String>,
	normalize_embeddings: bool,
	approximate_gelu: bool,
}

impl Default for BertConfig {
	fn default() -> Self {
		Self {
			model_id: Default::default(),
			revision: None,
			normalize_embeddings: true,
			approximate_gelu: false,
		}
	}
}
#[derive(Resource)]
pub struct Bert {
	config: BertConfig,
	model: BertModel,
	tokenizer: Tokenizer,
}

impl Bert {
	pub fn new(config: BertConfig) -> Result<Self> {
		let device = candle_core::Device::Cpu;
		let default_model =
			"sentence-transformers/all-MiniLM-L6-v2".to_string();
		let default_revision = "refs/pr/21".to_string();
		let (model_id, revision) =
			match (config.model_id.to_owned(), config.revision.to_owned()) {
				(Some(model_id), Some(revision)) => (model_id, revision),
				(Some(model_id), None) => (model_id, "main".to_string()),
				(None, Some(revision)) => (default_model, revision),
				(None, None) => (default_model, default_revision),
			};

		let repo = Repo::with_revision(model_id, RepoType::Model, revision);
		let api = Api::new()?;
		let api = api.repo(repo);
		let tokenizer_filename = api.get("tokenizer.json")?;
		let weights_filename = api.get("model.safetensors")?;
		let config_filename = api.get("config.json")?;
		let candle_config = std::fs::read_to_string(config_filename)?;
		let mut candle_config: Config = serde_json::from_str(&candle_config)?;
		let tokenizer =
			Tokenizer::from_file(tokenizer_filename).map_err(E::msg)?;

		let vb = unsafe {
			VarBuilder::from_mmaped_safetensors(
				&[weights_filename],
				DTYPE,
				&device,
			)?
		};
		if config.approximate_gelu {
			candle_config.hidden_act = HiddenAct::GeluApproximate;
		}
		let model = BertModel::load(vb, &candle_config)?;
		Ok(Self {
			config,
			model,
			tokenizer,
		})
	}

	/// Calculate the embeddings for a list of sentences
	/// For a trivial example this may take ~500ms
	pub fn get_embeddings(
		&mut self,
		options: Vec<Cow<'static, str>>,
	) -> Result<SentenceEmbeddings> {
		if let Some(pp) = self.tokenizer.get_padding_mut() {
			pp.strategy = tokenizers::PaddingStrategy::BatchLongest
		} else {
			let pp = PaddingParams {
				strategy: tokenizers::PaddingStrategy::BatchLongest,
				..Default::default()
			};
			self.tokenizer.with_padding(Some(pp));
		}
		let tokens = self
			.tokenizer
			.encode_batch(options.clone(), true)
			.map_err(E::msg)?;
		let token_ids = tokens
			.iter()
			.map(|tokens| {
				let tokens = tokens.get_ids().to_vec();
				Ok(Tensor::new(tokens.as_slice(), &self.model.device)?)
			})
			.collect::<Result<Vec<_>>>()?;

		let token_ids = Tensor::stack(&token_ids, 0)?;
		let token_type_ids = token_ids.zeros_like()?;
		let embeddings = self.model.forward(&token_ids, &token_type_ids)?;
		// Apply some avg-pooling by taking the mean embedding value for all tokens (including padding)
		let (_n_sentence, n_tokens, _hidden_size) = embeddings.dims3()?;
		let embeddings = (embeddings.sum(1)? / (n_tokens as f64))?;
		let embeddings = if self.config.normalize_embeddings {
			normalize_l2(&embeddings)?
		} else {
			embeddings
		};

		Ok(SentenceEmbeddings::new(options, embeddings))
	}


	pub fn prompt_tensor(
		&mut self,
		prompt: &str,
		iterations: usize,
	) -> Result<Vec<Tensor>> {
		let tokenizer = self
			.tokenizer
			.with_padding(None)
			.with_truncation(None)
			.map_err(E::msg)?;
		let tokens = tokenizer
			.encode(prompt, true)
			.map_err(E::msg)?
			.get_ids()
			.to_vec();
		let token_ids =
			Tensor::new(&tokens[..], &self.model.device)?.unsqueeze(0)?;
		let token_type_ids = token_ids.zeros_like()?;

		let tensors = (0..iterations)
			.map(|_| self.model.forward(&token_ids, &token_type_ids))
			.collect::<Result<Vec<_>, candle_core::Error>>()?;

		Ok(tensors)
	}
}

fn normalize_l2(v: &Tensor) -> Result<Tensor> {
	Ok(v.broadcast_div(&v.sqr()?.sum_keepdim(1)?.sqrt()?)?)
}



#[cfg(test)]
mod test {
	use crate::prelude::*;
	use anyhow::Result;
	use sweet::*;

	#[test]
	fn works() -> Result<()> {
		pretty_env_logger::try_init().ok();

		let mut bert = Bert::new(BertConfig::default())?;
		let embeddings = bert.get_embeddings(vec![
			"The cat sits outside".into(),
			"A man is playing guitar".into(),
			"I love pasta".into(),
			"The new movie is awesome".into(),
			"The cat plays in the garden".into(),
			"A woman watches TV".into(),
			"The new movie is so great".into(),
			"Do you like pizza?".into(),
		])?;

		let results = embeddings.scores(0)?;
		expect(embeddings.sentences[results[0].0].as_ref())
			.to_be("The cat plays in the garden")?;

		Ok(())
	}
}

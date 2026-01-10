use crate::ast::Type;
use crate::parser::Rule;
use pest::iterators::Pair;

pub(crate) fn parse_type(pair: Pair<Rule>) -> Result<Type, String> {
	let span = pair.as_span();
	let inner = pair
		.into_inner()
		.next()
		.ok_or_else(|| format!("Empty type at {:?}", span))?;

	match inner.as_rule() {
		Rule::basic_type => parse_basic_type(&inner),
		Rule::list_type => {
			let inner_type = inner.into_inner().next().unwrap();
			Ok(Type::List(Box::new(parse_basic_type(&inner_type)?)))
		}
		Rule::pairs_type => {
			let mut types = inner.into_inner();
			let key_type = parse_basic_type(&types.next().unwrap())?;
			let value_type = parse_basic_type(&types.next().unwrap())?;
			Ok(Type::Pairs(Box::new(key_type), Box::new(value_type)))
		}
		_ => Err("Unexpected type rule".to_string()),
	}
}

pub(crate) fn parse_basic_type(pair: &Pair<Rule>) -> Result<Type, String> {
	match pair.as_str() {
		"Int" => Ok(Type::Int),
		"Float" => Ok(Type::Float),
		"String" => Ok(Type::String),
		"Boolean" | "Bool" => Ok(Type::Boolean),
		"Array" => Ok(Type::Array),
		"Object" => Ok(Type::Object),
		"Regex" => Ok(Type::Regex),
		_ => Err(format!("Unknown type: {}", pair.as_str())),
	}
}

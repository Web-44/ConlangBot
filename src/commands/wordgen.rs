use std::collections::HashMap;
use rand::Rng;
use rand::rngs::ThreadRng;
use serenity::all::{CommandData, CommandInteraction, CommandOptionType, Context, CreateCommand};
use serenity::builder::{CreateAttachment, CreateCommandOption, EditInteractionResponse};

pub fn register() -> CreateCommand {
    let mut cmd = CreateCommand::new("wordgen")
        .description("Genarate words based on provided syllables")
        .add_option(CreateCommandOption::new(CommandOptionType::Integer, "amount", "How many words to generate")
            .min_int_value(1)
            .max_int_value(1000)
            .required(true))
        .add_option(CreateCommandOption::new(CommandOptionType::Integer, "min-syllables", "The minimum amount of syllables in a word")
            .min_int_value(1)
            .max_int_value(50)
            .required(true))
        .add_option(CreateCommandOption::new(CommandOptionType::Integer, "max-syllables", "The maximum amount of syllables in a word")
            .min_int_value(1)
            .max_int_value(50)
            .required(true))
        .add_option(CreateCommandOption::new(CommandOptionType::String, "syllable", "A list of syllables that can be constructed. Example: CVC,CV(V)(C(!)),C(VC(VVC))V")
            .required(true));

    for i in 1..=20 {
        cmd = cmd.add_option(CreateCommandOption::new(CommandOptionType::String, &format!("category-{}", i), "A syllable category. Example: V:a,e,i,o,u")
            .min_length(3)
            .max_length(150)
            .required(false));
    }

    cmd
}

pub async fn run(ctx: &Context, cmd: CommandInteraction) {
    let _ = cmd.defer(&ctx).await;

    match wordgen(&cmd.data).await {
        Ok(words) => {
            let _ = cmd.edit_response(&ctx, EditInteractionResponse::new()
                .new_attachment(CreateAttachment::bytes(words.as_bytes(), "wordlist.txt"))).await;
        }
        Err(err) => {
            let _ = cmd.edit_response(&ctx, EditInteractionResponse::new()
                .content(err)).await;
        }
    }
}

async fn wordgen(data: &CommandData) -> Result<String, String> {
    let amount = data.options[0].value.as_i64().unwrap();
    let min_syllables = data.options[1].value.as_i64().unwrap();
    let max_syllables = data.options[2].value.as_i64().unwrap();
    let syllable = data.options[3].value.as_str().unwrap();

    let mut categories = HashMap::new();
    for i in 1..=20 {
        if let Some(option) = data.options.get(i + 3) {
            let definition = option.value.as_str().unwrap();
            if let Some((name, letters)) = definition.split_once(":") {
                if name.len() != 1 {
                    return Err(format!("Only single-character names are supported, but category {i} does not match that"));
                }

                let letters: Vec<&str> = letters.split(",").collect();
                categories.insert(name.chars().next().unwrap(), letters);
            } else {
                return Err(format!("Category {i} is not formatted correctly. Example: V:a,e,i,o,u"));
            }
        }
    }

    for char in syllable.chars() {
        if char == ',' || char == '(' || char == ')' || char == '!' {
            continue;
        }
        if !categories.contains_key(&char) {
            return Err(format!("Category not defined: {char}"));
        }
    }

    let mut rng = rand::thread_rng();

    let mut words = Vec::with_capacity(amount as usize);
    for _ in 0..amount {
        for _ in 0..10 {
            let word = generate_word(&mut rng, min_syllables as usize, max_syllables as usize, syllable, &categories);
            if !words.contains(&word) {
                words.push(word);
                break;
            }
        }
    }

    let longest_word = words.iter().map(|word| word.len()).max().unwrap_or(0) + 2;

    let mut result = String::new();
    for (idx, word) in words.into_iter().enumerate() {
        result.push_str(word.as_str());
        if idx % 5 == 4 {
            result.push_str("\n");
        } else {
            result.push_str(" ".repeat(longest_word - word.len()).as_str());
        }
    }
    Ok(result)
}

fn generate_word(rng: &mut ThreadRng, min_syllables: usize, max_syllables: usize,
                       syllable: &str, categories: &HashMap<char, Vec<&str>>) -> String {
    let mut word = String::new();
    let syllable_count = rng.gen_range(min_syllables..=max_syllables);

    for _ in 0..syllable_count {
        let syllable: Vec<&str> = syllable.split(",").collect();
        let syllable = syllable.get(rng.gen_range(0..syllable.len())).unwrap();

        let mut skipping = 0u8;
        for char in syllable.chars() {
            if char == '(' {
                if skipping > 0 || rng.gen() {
                    skipping += 1;
                }
            } else if char == ')' {
                if skipping > 0 {
                    skipping -= 1;
                }
            } else if skipping == 0 {
                if char == '!' {
                    if let Some(last) = word.chars().last() {
                        word.push(last);
                    }
                } else {
                    let letters = categories.get(&char).unwrap();
                    let letter = letters.get(rng.gen_range(0..letters.len())).unwrap();
                    word.push_str(letter);
                }
            }
        }
    }

    word
}
use std::env::args;
use chatgpt::prelude::*;
use crossterm::{
    terminal::{
        SetSize,
        size,
        ScrollUp,
        ScrollDown,
        EnableLineWrap,
        Clear,
        ClearType, EnterAlternateScreen
    },
    style::{Print},
    execute
};
use futures_util::StreamExt;
use std::io::{stdout, Write};
mod markdown_highlighter;


static mut BUFFER: String = String::new();
fn update_buffer(new_string: String) {
    unsafe {
        BUFFER.push_str(new_string.as_str());
        let mut parsed_buffer = markdown_highlighter::parse_markdown(BUFFER.as_str());
        parsed_buffer += "\n";
        execute!(
            stdout(),
            Clear(ClearType::All),
            Print(parsed_buffer),
            ).unwrap();
    }
}


async fn get_response(query :String, client: ChatGPT) {
    let stream = client
        .send_message_streaming(query.as_str())
        .await;

    // Iterating over stream contents
    stream.unwrap().for_each(|each| async move {
        match each {
            ResponseChunk::Content { delta, response_index: _,} => {
                update_buffer(delta);
            }
            _ => {}
        }
    })
    .await;
    unsafe {
        BUFFER.push_str("\n\n");
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Creating a client
    let key = args().nth(1).unwrap();
    let client = ChatGPT::new(key)?;
    let colums = size().unwrap().0;
    let _initial_space = "\n".repeat(colums as usize);
    execute!(
        stdout(),
        EnterAlternateScreen,
        EnableLineWrap,
        Print(_initial_space),
        Clear(ClearType::All),
        ).unwrap();
    let mut go_on = true;
    while go_on {
        let mut input = String::new();
        execute!(
            stdout(),
            Print("\nENTER PROMPT: \n")
            ).unwrap();
        std::io::stdin().read_line(&mut input).unwrap();
        if input.trim() == "gpt exit" {
            go_on = false;
        } else {
            unsafe {
                BUFFER.push_str("\nYOU: \n\n");
                BUFFER.push_str(input.as_str());
                BUFFER.push_str("\nGPT: \n\n");
            }
            get_response(input, client.clone()).await;
        }
    }
    Ok(())
}

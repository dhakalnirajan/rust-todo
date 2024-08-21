/*

Todo List CLI Application

=====================================
Overview

This is a simple Todo List CLI application written in Rust. It allows users to add, complete, delete, and list todo items. The application stores its data in a JSON file named "todo.json".
Usage
Commands

The application supports the following commands:

    add: Adds a new todo item to the list.
    complete: Marks a todo item as complete.
    delete: Deletes a todo item from the list.
    list: Lists all todo items.

Command Line Arguments

The application expects the following command line arguments:

    todo <command> [args]

For example:

    todo add "Buy milk" "PENDING"
    todo complete 0
    todo delete 1
    todo list


*/

#![crate_type = "cdylib"]
use std::fs;
use std::env;
use std::process;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct TodoItem {
    title: String,
    status: String,
}


#[derive(Serialize, Deserialize, Debug)]
// Represents the entire todo list, containing two vectors: pending and done.
struct TodoList {
    pending: Vec<TodoItem>,
    done: Vec<TodoItem>,
}

impl TodoList {
    fn new() -> TodoList {
        TodoList {
            pending: Vec::new(),
            done: Vec::new(),
        }
    }

    fn insert(&mut self, item: TodoItem) {
        self.pending.push(item);
    }

    fn complete(&mut self, index: usize) -> Result<(), String> {
        if index >= self.pending.len() {
            return Err("Invalid index".to_string());
        }

        let mut item = self.pending.remove(index);
        item.status = "DONE".to_string();
        self.done.push(item);

        Ok(())
    }

    fn delete(&mut self, index: usize) -> Result<(), String> {
        if index >= self.pending.len() {
            return Err("Invalid index".to_string());
        }

        self.pending.remove(index);

        Ok(())
    }

    fn list(&self) {
        println!("Pending tasks:");
        for (index, item) in self.pending.iter().enumerate() {
            println!("{}: {}", index, item.title);
        }

        println!("Done tasks:");
        for (index, item) in self.done.iter().enumerate() {
            println!("{}: {}", index, item.title);
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: todo <command> [args]");
        process::exit(1);
    }

    let command = &args[1];

    let mut todo_list = match fs::read_to_string("todo.json") {
        Ok(data) => serde_json::from_str(&data).unwrap_or(TodoList::new()),
        Err(_) => TodoList::new(),
    };

    match command.as_str() {
        "add" => {
            if args.len() < 4 {
                println!("Usage: todo add <title> <status>");
                process::exit(1);
            }

            let title = &args[2];
            let status = &args[3];

            todo_list.insert(TodoItem {
                title: title.to_string(),
                status: status.to_string(),
            });
        },
        "complete" => {
            if args.len() < 3 {
                println!("Usage: todo complete <index>");
                process::exit(1);
            }

            let index = match args[2].parse::<usize>() {
                Ok(index) => index,
                Err(_) => {
                    println!("Invalid index");
                    process::exit(1);
                },
            };

            match todo_list.complete(index) {
                Ok(_) => (),
                Err(err) => {
                    println!("{}", err);
                    process::exit(1);
                },
            }
        },
        "delete" => {
            if args.len() < 3 {
                println!("Usage: todo delete <index>");
                process::exit(1);
            }

            let index = match args[2].parse::<usize>() {
                Ok(index) => index,
                Err(_) => {
                    println!("Invalid index");
                    process::exit(1);
                },
            };

            match todo_list.delete(index) {
                Ok(_) => (),
                Err(err) => {
                    println!("{}", err);
                    process::exit(1);
                },
            }
        },
        "list" => {
            todo_list.list();
        },
        _ => {
            println!("Unknown command");
            process::exit(1);
        },
    }

    fs::write("todo.json", serde_json::to_string(&todo_list).unwrap()).unwrap();
}

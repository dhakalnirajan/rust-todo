use gtk::prelude::*;
use std::fs::File;
use std::io::Write;
use std::process::Command;

mod todo_list {
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct TodoItem {
        pub title: String,
        pub status: String,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct TodoList {
        pub pending: Vec<TodoItem>,
        pub done: Vec<TodoItem>,
    }

    impl TodoList {
        pub fn new() -> TodoList {
            TodoList {
                pending: Vec::new(),
                done: Vec::new(),
            }
        }

        pub fn insert(&mut self, item: TodoItem) {
            self.pending.push(item);
        }

        pub fn complete(&mut self, index: usize) -> Result<(), String> {
            if index >= self.pending.len() {
                return Err("Invalid index".to_string());
            }

            let mut item = self.pending.remove(index);
            item.status = "DONE".to_string();
            self.done.push(item);

            Ok(())
        }

        pub fn delete(&mut self, index: usize) -> Result<(), String> {
            if index >= self.pending.len() {
                return Err("Invalid index".to_string());
            }

            self.pending.remove(index);

            Ok(())
        }
    }
}

fn update_ui(todo_list: &todo_list::TodoList, ui: &gtk::ApplicationWindow) {
    let pending_listbox = ui.lookup_widget::<gtk::ListBox>("pending_listbox").unwrap();
    let done_listbox = ui.lookup_widget::<gtk::ListBox>("done_listbox").unwrap();

    pending_listbox.foreach(|widget| {
        pending_listbox.remove(&widget.downcast::<gtk::Label>().unwrap());
    });

    done_listbox.foreach(|widget| {
        done_listbox.remove(&widget.downcast::<gtk::Label>().unwrap());
    });

    for (index, item) in todo_list.pending.iter().enumerate() {
        let label = gtk::Label::new(Some(&format!("{}: {}", index, item.title)));
        pending_listbox.insert(&label, -1);
    }

    for (index, item) in todo_list.done.iter().enumerate() {
        let label = gtk::Label::new(Some(&format!("{}: {}", index, item.title)));
        done_listbox.insert(&label, -1);
    }
}

fn build_ui(application: &gtk::Application) {
    let glade_src = include_str!("todo_ui.glade");
    let glade_bytes = glade_src.as_bytes();
    let glade_buf = std::io::Cursor::new(glade_bytes);
    let ui = gtk::Builder::from_glade(glade_buf);

    let window: gtk::ApplicationWindow = ui.get_object("main_window").unwrap();
    window.set_application(Some(application));

    let todo_list = match File::open("todo.json") {
        Ok(file) => {
            let todo_list: todo_list::TodoList = serde_json::from_reader(file).unwrap_or(todo_list::TodoList::new());
            todo_list
        }
        Err(_) => todo_list::TodoList::new(),
    };

    update_ui(&todo_list, &window);

    let add_button = ui.get_object("add_button").unwrap();
    let add_entry = ui.get_object("add_entry").unwrap();
    let title_entry = ui.get_object("title_entry").unwrap();
    let status_entry = ui.get_object("status_entry").unwrap();
    let pending_listbox = ui.get_object("pending_listbox").unwrap();
    let done_listbox = ui.get_object("done_listbox").unwrap();

    let add_button_clone = add_button.clone();
    let todo_list_clone = todo_list.clone();
    add_button.connect_clicked(move |_| {
        let title = title_entry.get_text().unwrap_or_else(|| String::from("Untitled"));
        let status = status_entry.get_text().unwrap_or_else(|| String::from("PENDING"));
        let mut todo_list = todo_list_clone.clone();

        todo_list.insert(todo_list::TodoItem {
            title: title.clone(),
            status: status.clone(),
        });

        title_entry.set_text("");
        status_entry.set_text("");

        let todo_json = serde_json::to_string(&todo_list).unwrap();
        let mut file = File::create("todo.json").unwrap();
        file.write_all(todo_json.as_bytes()).unwrap();

        update_ui(&todo_list, &window);

        add_button_clone.grab_focus();
    });

    let complete_button = ui.get_object("complete_button").unwrap();
    let complete_button_clone = complete_button.clone();
    let todo_list_clone = todo_list.clone();
    complete_button.connect_clicked(move |_| {
        let selected = pending_listbox.selected_row();
        if let Some(path) = selected {
            let index = path.idx as usize;
            let mut todo_list = todo_list_clone.clone();

            match todo_list.complete(index) {
                Ok(_) => {
                    let todo_json = serde_json::to_string(&todo_list).unwrap();
                    let mut file = File::create("todo.json").unwrap();
                    file.write_all(todo_json.as_bytes()).unwrap();

                    update_ui(&todo_list, &window);
                }
                Err(err) => {
                    println!("Error: {}", err);
                }
            }
        }

        complete_button_clone.grab_focus();
    });

    let delete_button = ui.get_object("delete_button").unwrap();
    let delete_button_clone = delete_button.clone();
    let todo_list_clone = todo_list.clone();
    delete_button.connect_clicked(move |_| {
        let selected = pending_listbox.selected_row();
        if let Some(path) = selected {
            let index = path.idx as usize;
            let mut todo_list = todo_list_clone.clone();

            match todo_list.delete(index) {
                Ok(_) => {
                    let todo_json = serde_json::to_string(&todo_list).unwrap();
                    let mut file = File::create("todo.json").unwrap();
                    file.write_all(todo_json.as_bytes()).unwrap();

                    update_ui(&todo_list, &window);
                }
                Err(err) => {
                    println!("Error: {}", err);
                }
            }
        }

        delete_button_clone.grab_focus();
    });

    window.show_all();
}

fn main() {
    if gtk::init().is_err() {
        eprintln!("Failed to initialize GTK.");
        return;
    }

    let application = gtk::Application::builder()
        .application_id("com.example.TodoApp")
        .build();

    application.connect_activate(|app| {
        build_ui(app);
    });

    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        let command = &args[1];

        match command.as_str() {
            "add" => {
                let title = if args.len() > 2 {
                    args[2].clone()
                } else {
                    String::from("Untitled")
                };

                let status = if args.len() > 3 {
                    args[3].clone()
                } else {
                    String::from("PENDING")
                };

                let mut todo_list = match File::open("todo.json") {
                    Ok(file) => {
                        let todo_list: todo_list::TodoList = serde_json::from_reader(file).unwrap_or(todo_list::TodoList::new());
                        todo_list
                    }
                    Err(_) => todo_list::TodoList::new(),
                };

                todo_list.insert(todo_list::TodoItem { title, status });

                let todo_json = serde_json::to_string(&todo_list).unwrap();
                let mut file = File::create("todo.json").unwrap();
                file.write_all(todo_json.as_bytes()).unwrap();
            }
            "complete" => {
                if args.len() < 3 {
                    eprintln!("Usage: todo complete <index>");
                    return;
                }

                let index: usize = args[2].parse().unwrap();

                let mut todo_list = match File::open("todo.json") {
                    Ok(file) => {
                        let todo_list: todo_list::TodoList = serde_json::from_reader(file).unwrap_or(todo_list::TodoList::new());
                        todo_list
                    }
                    Err(_) => todo_list::TodoList::new(),
                };

                match todo_list.complete(index) {
                    Ok(_) => {
                        let todo_json = serde_json::to_string(&todo_list).unwrap();
                        let mut file = File::create("todo.json").unwrap();
                        file.write_all(todo_json.as_bytes()).unwrap();
                    }
                    Err(err) => {
                        eprintln!("Error: {}", err);
                    }
                }
            }
            "delete" => {
                if args.len() < 3 {
                    eprintln!("Usage: todo delete <index>");
                    return;
                }

                let index: usize = args[2].parse().unwrap();

                let mut todo_list = match File::open("todo.json") {
                    Ok(file) => {
                        let todo_list: todo_list::TodoList = serde_json::from_reader(file).unwrap_or(todo_list::TodoList::new());
                        todo_list
                    }
                    Err(_) => todo_list::TodoList::new(),
                };

                match todo_list.delete(index) {
                    Ok(_) => {
                        let todo_json = serde_json::to_string(&todo_list).unwrap();
                        let mut file = File::create("todo.json").unwrap();
                        file.write_all(todo_json.as_bytes()).unwrap();
                    }
                    Err(err) => {
                        eprintln!("Error: {}", err);
                    }
                }
            }
            _ => {
                eprintln!("Unknown command");
            }
        }
    }

    application.run();
}

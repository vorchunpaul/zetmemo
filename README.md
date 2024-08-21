# Telegram Note-Taking Bot with AI-Generated Summaries and Tags

## Overview

This project is a Telegram bot written in Rust that allows users to quickly add notes. The bot is designed to enhance the note-taking experience by automatically generating short descriptions (up to 80 characters) and relevant tags for each note using a neural network. Additionally, the bot supports advanced features inspired by the Zettelkasten method, enabling users to analyze their thoughts and explore connections between their ideas.

## Features

- **Quick Note Addition**: Users can easily add notes via Telegram with minimal effort.
- **AI-Generated Summaries**: Each note is automatically summarized in 80 characters using a neural network.
- **Tag Generation**: The neural network also generates relevant tags for easy categorization and retrieval.
- **Zettelkasten-Inspired Analysis**: Users can explore connections between their notes and ideas, facilitating deep analysis and knowledge building.

## Installation

1. **Clone the repository**:
   ```sh
   git clone https://github.com/vorchunpaul/zetmemo.git
   cd zetmemo
   ```

2. **Install Rust**: Make sure you have Rust installed. You can download it [here](https://www.rust-lang.org/tools/install).

3. **Set up dependencies**:
   ```sh
   cargo build
   ```
   
4. **Run the bot**:
   ```sh
   cargo run
   ```

## Usage

1. **Start the bot**: Use the command `/start` in Telegram to begin interacting with the bot.
2. **Add a note**: Simply send a message with your note. The bot will automatically generate a summary and tags.

## Future Development

- **Web interface**: Develop a web interface to complement the Telegram bot for easier note management.

## Contributing

Contributions are welcome! Please feel free to submit a pull request or open an issue on GitHub.

## Contact

For any questions or suggestions, please contact us at [telegram](https://t.me/vorchunpaul).

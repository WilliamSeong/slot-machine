0. Prerequisites
This installation guide assumes you have Rust and Cargo already installed on your system. If you need to install Rust, visit https://www.rust-lang.org/tools/install.

You can verify Rust and Cargo are installed by running:
$ rustc --version
$ cargo --version


0.5 SQLite
Make sure sqlite is installed in order to run the application.
Visit, https://sqlite.org/download.html.

1. Download the Project
Download or clone the casino application project files to your computer.
Move the files to your desired location.

2. Navigate to Project Directory
Open a terminal or command prompt and navigate to the project root directory:
$ cd path/to/slot-machine


3. Build and Run the Application
Run the following command to build and start the application:
$ cargo run

This command will:
Automatically download and compile all required dependencies
Build the application executable
Launch the casino application
Note: The first run may take a while as Cargo downloads and compiles all dependencies.

4. Verify Installation
If installation was successful, you should see the Casino Login menu with three options:
 ═══ 🎰 Casino Login 🎰 ═══ 
Register
Sign In
Exit

Note: On first run, the application will automatically create the necessary database, .env, and logs files in the project directory.
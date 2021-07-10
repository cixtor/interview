# Interview

`interview` is a command line interface (CLI) originally written in Go (golang) but later re-written in Rust for learning purposes. The program offers a few commands to create, find, and open interview notes as part of my efforts to maintain an archive of all my interactions with hiring managers, technical recruiters, and staffing agencies.

The program creates new interview notes using the Electronic Mail Format (EML).

An EML file is an email message saved by an email application, such as Microsoft Outlook or Apple Mail. It contains the content of the message, along with the subject, sender, recipient(s), and date of the message. EML files may also store one or more email attachments, which are files sent with the message, and in this specific case, a job description in Markdown format.

EML file format specifications are available as per [RFC 822](http://www.ietf.org/rfc/rfc0822.txt) Standard Format.

## Installation

For safety reasons I do not provide compiled binaries of this program. However, you can build your own version of the program if you download a copy of the code and use a recent version of the Rust compiler to create a binary that is safe to use in your own machine.

1. Download the code from this repository
2. Execute this command `cargo build --release`
3. Move the resulting binary into your system `$PATH`
4. Run the program like so `interview [CompanyName]`

## Usage

Please use `interview help` to see a list of available options.

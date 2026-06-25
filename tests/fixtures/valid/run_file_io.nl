fn main -> string
    write_file("target/nous_fixture_io.txt", "alpha")
    append_file("target/nous_fixture_io.txt", " beta")
    read_file("target/nous_fixture_io.txt")

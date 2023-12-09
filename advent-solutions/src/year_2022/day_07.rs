use std::{
    collections::{BTreeMap, HashMap},
    fmt,
    io::BufRead,
    str::FromStr,
};

/// Explicit typing for the lines of a terminal session. Lines are either a `Command` or an
/// `Output` from one.
#[derive(Clone, Debug, PartialEq, Eq)]
enum TerminalLine {
    Command(String),
    Output(String),
}

impl TerminalLine {
    /// Returns `true` if the `TerminalLine` is a `Command` value
    fn _is_command(&self) -> bool {
        matches!(self, Self::Command(_))
    }

    /// Returns `true` if the `TerminalLine` is an `Output` value
    fn is_output(&self) -> bool {
        matches!(self, Self::Output(_))
    }

    /// Quick access to the trimmed source line this type was created with
    fn raw_line(&self) -> &str {
        match self {
            Self::Command(s) | Self::Output(s) => s,
        }
    }
}

impl FromStr for TerminalLine {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            Err("Terminal output should not have empty lines")
        } else if s.starts_with('$') {
            Ok(Self::Command(s[2..].to_owned()))
        } else {
            Ok(Self::Output(s.to_owned()))
        }
    }
}

/// Type representation for the elements of a path as expressed in a terminal borrowing text where
/// necessary.
#[derive(Clone, Debug, PartialEq, Eq)]
enum PathSegment<'a> {
    Root,
    Up,
    Down(&'a str),
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Path(String);

impl Path {
    /// Turns the raw text representation of a path into individual segments for easier tree
    /// traversal. A `Path` starting with `/` will first output a `PathSegment::Root` to indicate
    /// navigation to the root of the file system. The remaining parts are parsed as either moving
    /// up a directory or down into another directory.
    fn segments(&self) -> impl Iterator<Item = Result<PathSegment, &'static str>> {
        let source = if self.0.ends_with('/') {
            &self.0[..(self.0.len() - 1)]
        } else {
            &self.0
        };

        source.split('/').enumerate().map(|es| match es {
            (0, "") => Ok(PathSegment::Root),
            (_, "") => Err("path should consist of valid segments"),
            (_, "..") => Ok(PathSegment::Up),
            (_, s) => Ok(PathSegment::Down(s)),
        })
    }
}

/// Information gathered when calling `ls` in a directory. Items listed may be directory or a
/// filename with its size in bytes.
#[derive(Clone, Debug, PartialEq, Eq)]
enum StatEntry {
    Directory(String),
    File(String, usize),
}

impl StatEntry {
    /// Get the file system name of either the `Directory` or `File` this `StatEntry` represents.
    fn name(&self) -> &str {
        match self {
            StatEntry::Directory(name) | StatEntry::File(name, _) => name,
        }
    }
}

impl FromStr for StatEntry {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.split_once(' ') {
            Some(("dir", name)) => Ok(StatEntry::Directory(name.to_owned())),
            Some((size_bytes, name)) => {
                if let Ok(size_bytes) = size_bytes.parse() {
                    Ok(StatEntry::File(name.to_owned(), size_bytes))
                } else {
                    Err("File entries should be a number (size in bytes) followed by a filename")
                }
            }
            None => Err("Stat entry should always be two space-separated parts"),
        }
    }
}

/// Represents the commands that this problem supports:
/// - `Jump`ing to another path
/// - `List` of items in the current directory
#[derive(Clone, Debug, PartialEq, Eq)]
enum Command {
    Jump(Path),
    List(Vec<StatEntry>),
}

impl Command {
    /// Returns `true` if the `Command` is a `Jump` value
    fn _is_jump(&self) -> bool {
        matches!(self, Self::Jump(_))
    }

    /// Returns `true` if the `Command` is a `List` value
    fn _is_list(&self) -> bool {
        matches!(self, Self::List(_))
    }
}

impl FromStr for Command {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match &s[..2] {
            "cd" => Ok(Self::Jump(Path(s[3..].to_owned()))),
            "ls" => Ok(Self::List(Vec::new())),
            _ => Err("Unknown command encountered"),
        }
    }
}

/// Performs nearly identically to `TakeWhile` but allows for an additional method that receives
/// the first item for which `predicate` returns false.
#[derive(Clone, Debug)]
struct TakeWhileLossless<I, P, U> {
    source: I,
    predicate: P,
    first_false: U,
    exhausted: bool,
}

impl<I: Iterator, P, U> Iterator for TakeWhileLossless<I, P, U>
where
    P: FnMut(&I::Item) -> bool,
    U: FnMut(I::Item),
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        if self.exhausted {
            None
        } else {
            let v = self.source.next()?;

            if (self.predicate)(&v) {
                Some(v)
            } else {
                self.exhausted = true;

                (self.first_false)(v);

                None
            }
        }
    }
}

trait TakeWhileLosselssExt: Iterator {
    /// Very similar to `take_while` but calls `first_false` with first item for which `predicate`
    /// returns false so that it will not be inadvertently discarded.
    fn take_while_lossless<P, U>(
        self,
        predicate: P,
        first_false: U,
    ) -> TakeWhileLossless<Self, P, U>
    where
        Self: Sized,
    {
        TakeWhileLossless {
            source: self,
            predicate,
            first_false,
            exhausted: false,
        }
    }
}

impl<T> TakeWhileLosselssExt for T where T: Iterator {}

/// Seamlessly associate commands with their output and bundle them into a `Command`. The builder
/// will consume the iterator until it can create a `Command` and return it.
#[derive(Clone, Debug)]
struct CommandBuilder<I>
where
    I: Iterator<Item = TerminalLine>,
{
    source: I,
    extra: Option<I::Item>,
}

impl<I> Iterator for CommandBuilder<I>
where
    I: Iterator<Item = TerminalLine>,
{
    type Item = Command;

    fn next(&mut self) -> Option<Self::Item> {
        self.extra
            .take()
            .or_else(|| self.source.next())
            .map(|tl| {
                if let TerminalLine::Command(s) = tl {
                    s.parse::<Command>()
                        .expect("command should be either ls or cd")
                } else {
                    panic!("found output when expecting a command")
                }
            })
            .map(|mut c| {
                if let Command::List(ref mut v) = &mut c {
                    v.extend(
                        (&mut self.source)
                            .take_while_lossless(TerminalLine::is_output, |tl| {
                                self.extra = Some(tl);
                            })
                            .map(|tl| {
                                tl.raw_line().parse().expect(
                                "output lines following ls should all be file/directory details",
                            )
                            }),
                    );
                }

                c
            })
    }
}

trait Commands: Iterator<Item = TerminalLine> {
    /// Enables recovering structured `Command`s with their outputs from a sequence of raw
    /// `TerminalLine`s.
    fn commands(self) -> CommandBuilder<Self>
    where
        Self: Sized,
    {
        CommandBuilder {
            source: self,
            extra: None,
        }
    }
}

impl<T> Commands for T where T: Iterator<Item = TerminalLine> {}

/// Details about a directory in a `FileSystem` including its `name` and a name to index mapping
/// for the `children` of this directory.
#[derive(Clone, Debug, PartialEq, Eq)]
struct DirectoryEntry {
    name: String,
    children: HashMap<String, usize>,
}

/// Details about a file in a `FileSystem` including its full `name` and the size of the file in
/// bytes via `size_bytes`.
#[derive(Clone, Debug, PartialEq, Eq)]
struct FileEntry {
    name: String,
    size_bytes: usize,
}

/// This problem's `FileSystem` consists of only two kinds of items:
/// - `Directory`: which has a name and can indirectly contain other `FileSystemEntry` items
/// - `Flie`: which has a name and a size in bytes
#[derive(Clone, Debug, PartialEq, Eq)]
enum FileSystemEntry {
    Directory(DirectoryEntry),
    File(FileEntry),
}

impl FileSystemEntry {
    /// Returns `true` if the `FileSystemEntry` is a `Directory` value
    fn is_directory(&self) -> bool {
        matches!(self, FileSystemEntry::Directory(_))
    }

    /// Returns `true` if the `FileSystemEntry` is a `File` value
    fn _is_file(&self) -> bool {
        matches!(self, FileSystemEntry::File(_))
    }

    fn name(&self) -> &str {
        match self {
            FileSystemEntry::Directory(d) => &d.name,
            FileSystemEntry::File(f) => &f.name,
        }
    }
}

impl From<&StatEntry> for FileSystemEntry {
    fn from(value: &StatEntry) -> Self {
        match value {
            StatEntry::Directory(name) => Self::Directory(DirectoryEntry {
                name: name.clone(),
                children: HashMap::default(),
            }),
            StatEntry::File(name, size_bytes) => Self::File(FileEntry {
                name: name.clone(),
                size_bytes: *size_bytes,
            }),
        }
    }
}

/// Trait for a type to visit all the entries in a `FileSystem` treating files and directories
/// distinctly.
trait FileSystemVisitor<'a> {
    fn visit_directory(&mut self, idx: usize, entry: &'a DirectoryEntry);

    fn visit_file(&mut self, idx: usize, entry: &'a FileEntry);
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct FileSystem(Vec<FileSystemEntry>);

impl FileSystem {
    /// Get the total number of `FileSystemEntry` objects in this `FileSystem`.
    fn len(&self) -> usize {
        self.0.len()
    }

    /// Visit every `FileSystemEntry` in this `FileSystem` in a depth-first order exactly once.
    /// Files are guaranteed to be visited before the directories that contain them. Calls the
    /// appropriate `FileSystemVisitor` method for the `FileSystemEntry` being observed. The visit
    /// order of siblings within a directory is not guaranteed.
    fn visit_depth_first<'a, V: FileSystemVisitor<'a>>(&'a self, visitor: &mut V)
    where
        Self: 'a,
    {
        let mut stack = Vec::with_capacity(self.0.len());

        stack.push((0usize, false));

        while let Some((idx, visited_children)) = stack.pop() {
            match &self.0[idx] {
                FileSystemEntry::Directory(details) => {
                    if visited_children {
                        visitor.visit_directory(idx, details);
                    } else {
                        stack.push((idx, true));

                        stack.extend(details.children.values().map(|ci| (*ci, false)));
                    }
                }
                FileSystemEntry::File(details) => visitor.visit_file(idx, details),
            }
        }
    }
}

impl FromIterator<Command> for FileSystem {
    fn from_iter<T: IntoIterator<Item = Command>>(iter: T) -> Self {
        let mut nodes: Vec<usize> = Vec::new();
        let mut current = 0usize;
        let mut fs = vec![FileSystemEntry::Directory(DirectoryEntry {
            name: "/".into(),
            children: HashMap::default(),
        })];

        for cmd in iter {
            match cmd {
                Command::Jump(path) => {
                    for seg in path.segments() {
                        match seg.expect("path segments should all be valid") {
                            PathSegment::Root => {
                                nodes.clear();
                                current = 0;
                            }
                            PathSegment::Up => {
                                current = nodes.pop().unwrap_or(0);
                            }
                            PathSegment::Down(d) => {
                                if let FileSystemEntry::Directory(entry) = &fs[current] {
                                    nodes.push(current);
                                    current = entry.children[d];
                                } else {
                                    panic!("Trying to change directories when not in a directory");
                                }
                            }
                        }
                    }
                }
                Command::List(entries) => {
                    let base_idx = fs.len();

                    if fs[current].is_directory() {
                        fs.extend(entries.iter().map(FileSystemEntry::from));

                        if let FileSystemEntry::Directory(d) = &mut fs[current] {
                            d.children.extend(
                                entries
                                    .iter()
                                    .enumerate()
                                    .map(|(i, e)| (e.name().to_owned(), base_idx + i)),
                            );
                        }
                    } else {
                        panic!("Trying to add files when not in a directory");
                    }
                }
            }
        }

        Self(fs)
    }
}

impl fmt::Display for FileSystem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut stack = Vec::with_capacity(self.0.len());
        stack.push((0, &self.0[0]));

        while let Some((depth, entry)) = stack.pop() {
            write!(f, "{1:>0$} {2}", depth * 2 + 1, '-', entry.name())?;

            match entry {
                FileSystemEntry::Directory(de) => {
                    writeln!(f, " (dir)")?;

                    // Briefly cast to BTreeMap to get consistent output
                    stack.extend(
                        de.children
                            .iter()
                            .collect::<BTreeMap<_, _>>()
                            .iter()
                            .rev()
                            .map(|(_, &&idx)| (depth + 1, &self.0[idx])),
                    );
                }
                FileSystemEntry::File(fe) => {
                    writeln!(f, " (file, size={})", fe.size_bytes)?;
                }
            }
        }

        Ok(())
    }
}

/// Simplified entry for a `FileSystem` to unify relevant information while calculating space used.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct SizeCacheEntry<'a> {
    is_directory: bool,
    size_bytes: usize,
    name: &'a str,
}

/// Calculates the sizes of all directories and their descendents in a `FileSystem` with a unified
/// cache for all entries for quick lookup.
#[derive(Clone, Debug, PartialEq, Eq)]
struct DirectorySizer<'a>(Vec<SizeCacheEntry<'a>>);

impl<'a> DirectorySizer<'a> {
    /// Build a `DirectorySizer` tied to the lifetime of the passed `FileSystem`, visit all entries
    /// via the `FileSystemVisitor` trait and build up the internal size cache.
    fn for_file_system(fs: &'a FileSystem) -> Self {
        let mut ds = Self(vec![SizeCacheEntry::default(); fs.len()]);

        fs.visit_depth_first(&mut ds);

        ds
    }

    /// Sums the sizes of all directories in the associated `FileSystem` underneath `max` size.
    /// This will double-count directories contained by other directories that fit under `max`.
    fn sum_under(&self, max: usize) -> usize {
        self.0
            .iter()
            .filter_map(|e| {
                if e.is_directory && e.size_bytes < max {
                    Some(e.size_bytes)
                } else {
                    None
                }
            })
            .sum()
    }

    /// Finds the smallest directory to remove that would get the `FileSystem` underneath
    /// `max_used` if such a directory exists.
    fn smallest_to_get_under(&self, max_used: usize) -> Option<(usize, &'a str)> {
        let current_used = self.0[0].size_bytes;

        if current_used < max_used {
            None
        } else {
            let needed = current_used - max_used;

            self.0
                .iter()
                .filter_map(|e| {
                    if e.is_directory && e.size_bytes >= needed {
                        Some((e.size_bytes, e.name))
                    } else {
                        None
                    }
                })
                .min()
        }
    }
}

impl<'a> FileSystemVisitor<'a> for DirectorySizer<'a> {
    fn visit_directory(&mut self, idx: usize, entry: &'a DirectoryEntry) {
        let total = entry
            .children
            .iter()
            .map(|(_, &i)| self.0[i].size_bytes)
            .sum();

        let cached = &mut self.0[idx];

        cached.is_directory = true;
        cached.size_bytes = total;
        cached.name = entry.name.as_str();
    }

    fn visit_file(&mut self, idx: usize, entry: &'a FileEntry) {
        let cached = &mut self.0[idx];

        cached.is_directory = false;
        cached.size_bytes = entry.size_bytes;
        cached.name = entry.name.as_str();
    }
}

pub fn part_01(reader: Option<impl BufRead>) {
    let reader = reader.expect("data should be available for this problem");

    let fs = reader
        .lines()
        .flatten()
        .map(|l| {
            l.parse::<TerminalLine>()
                .expect("all input lines should be terminal lines")
        })
        .commands()
        .collect::<FileSystem>();

    let dir_sizer = DirectorySizer::for_file_system(&fs);

    println!(
        "Sum of all directories less than 100,000 in size: {}",
        dir_sizer.sum_under(100_000)
    );
}

pub fn part_02(reader: Option<impl BufRead>) {
    let reader = reader.expect("data should be available for this problem");

    let fs = reader
        .lines()
        .flatten()
        .map(|l| {
            l.parse::<TerminalLine>()
                .expect("all input lines should be terminal lines")
        })
        .commands()
        .collect::<FileSystem>();

    let dir_sizer = DirectorySizer::for_file_system(&fs);

    println!(
        "Smallest directory to delete to get to 30,000,000 bytes of free space: {:?}",
        dir_sizer.smallest_to_get_under(40_000_000)
    );
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_terminal() {
        let input = r"$ cd /
$ ls
dir a
14848514 b.txt
8504156 c.dat
dir d
$ cd a
$ ls
dir e
29116 f
2557 g
62596 h.lst
$ cd e
$ ls
584 i
$ cd ..
$ cd ..
$ cd d
$ ls
4060174 j
8033020 d.log
5626152 d.ext
7214296 k";

        let parsed = input
            .lines()
            .flat_map(TerminalLine::from_str)
            .collect::<Vec<_>>();

        let expected = vec![
            TerminalLine::Command("cd /".into()),
            TerminalLine::Command("ls".into()),
            TerminalLine::Output("dir a".into()),
            TerminalLine::Output("14848514 b.txt".into()),
            TerminalLine::Output("8504156 c.dat".into()),
            TerminalLine::Output("dir d".into()),
            TerminalLine::Command("cd a".into()),
            TerminalLine::Command("ls".into()),
            TerminalLine::Output("dir e".into()),
            TerminalLine::Output("29116 f".into()),
            TerminalLine::Output("2557 g".into()),
            TerminalLine::Output("62596 h.lst".into()),
            TerminalLine::Command("cd e".into()),
            TerminalLine::Command("ls".into()),
            TerminalLine::Output("584 i".into()),
            TerminalLine::Command("cd ..".into()),
            TerminalLine::Command("cd ..".into()),
            TerminalLine::Command("cd d".into()),
            TerminalLine::Command("ls".into()),
            TerminalLine::Output("4060174 j".into()),
            TerminalLine::Output("8033020 d.log".into()),
            TerminalLine::Output("5626152 d.ext".into()),
            TerminalLine::Output("7214296 k".into()),
        ];

        assert_eq!(parsed, expected);
    }

    #[test]
    fn rebuild_commands() {
        let input = vec![
            TerminalLine::Command("cd /".into()),
            TerminalLine::Command("ls".into()),
            TerminalLine::Output("dir a".into()),
            TerminalLine::Output("14848514 b.txt".into()),
            TerminalLine::Output("8504156 c.dat".into()),
            TerminalLine::Output("dir d".into()),
            TerminalLine::Command("cd a".into()),
            TerminalLine::Command("ls".into()),
            TerminalLine::Output("dir e".into()),
            TerminalLine::Output("29116 f".into()),
            TerminalLine::Output("2557 g".into()),
            TerminalLine::Output("62596 h.lst".into()),
            TerminalLine::Command("cd e".into()),
            TerminalLine::Command("ls".into()),
            TerminalLine::Output("584 i".into()),
            TerminalLine::Command("cd ..".into()),
            TerminalLine::Command("cd ..".into()),
            TerminalLine::Command("cd d".into()),
            TerminalLine::Command("ls".into()),
            TerminalLine::Output("4060174 j".into()),
            TerminalLine::Output("8033020 d.log".into()),
            TerminalLine::Output("5626152 d.ext".into()),
            TerminalLine::Output("7214296 k".into()),
        ];

        let parsed = input.into_iter().commands().collect::<Vec<_>>();

        let expected = vec![
            Command::Jump(Path("/".into())),
            Command::List(vec![
                StatEntry::Directory("a".into()),
                StatEntry::File("b.txt".into(), 14_848_514),
                StatEntry::File("c.dat".into(), 8_504_156),
                StatEntry::Directory("d".into()),
            ]),
            Command::Jump(Path("a".into())),
            Command::List(vec![
                StatEntry::Directory("e".into()),
                StatEntry::File("f".into(), 29116),
                StatEntry::File("g".into(), 2557),
                StatEntry::File("h.lst".into(), 62596),
            ]),
            Command::Jump(Path("e".into())),
            Command::List(vec![StatEntry::File("i".into(), 584)]),
            Command::Jump(Path("..".into())),
            Command::Jump(Path("..".into())),
            Command::Jump(Path("d".into())),
            Command::List(vec![
                StatEntry::File("j".into(), 4_060_174),
                StatEntry::File("d.log".into(), 8_033_020),
                StatEntry::File("d.ext".into(), 5_626_152),
                StatEntry::File("k".into(), 7_214_296),
            ]),
        ];

        assert_eq!(parsed, expected);
    }

    #[test]
    fn build_filesystem() {
        let input = vec![
            Command::Jump(Path("/".into())),
            Command::List(vec![
                StatEntry::Directory("a".into()),
                StatEntry::File("b.txt".into(), 14_848_514),
                StatEntry::File("c.dat".into(), 8_504_156),
                StatEntry::Directory("d".into()),
            ]),
            Command::Jump(Path("a".into())),
            Command::List(vec![
                StatEntry::Directory("e".into()),
                StatEntry::File("f".into(), 29116),
                StatEntry::File("g".into(), 2557),
                StatEntry::File("h.lst".into(), 62596),
            ]),
            Command::Jump(Path("e".into())),
            Command::List(vec![StatEntry::File("i".into(), 584)]),
            Command::Jump(Path("..".into())),
            Command::Jump(Path("..".into())),
            Command::Jump(Path("d".into())),
            Command::List(vec![
                StatEntry::File("j".into(), 4_060_174),
                StatEntry::File("d.log".into(), 8_033_020),
                StatEntry::File("d.ext".into(), 5_626_152),
                StatEntry::File("k".into(), 7_214_296),
            ]),
        ];

        let built = input.into_iter().collect::<FileSystem>();

        let expected = r"- / (dir)
  - a (dir)
    - e (dir)
      - i (file, size=584)
    - f (file, size=29116)
    - g (file, size=2557)
    - h.lst (file, size=62596)
  - b.txt (file, size=14848514)
  - c.dat (file, size=8504156)
  - d (dir)
    - d.ext (file, size=5626152)
    - d.log (file, size=8033020)
    - j (file, size=4060174)
    - k (file, size=7214296)
";

        assert_eq!(format!("{built}"), expected);
    }

    #[test]
    fn sum_dirs_100k() {
        let input = vec![
            Command::Jump(Path("/".into())),
            Command::List(vec![
                StatEntry::Directory("a".into()),
                StatEntry::File("b.txt".into(), 14_848_514),
                StatEntry::File("c.dat".into(), 8_504_156),
                StatEntry::Directory("d".into()),
            ]),
            Command::Jump(Path("a".into())),
            Command::List(vec![
                StatEntry::Directory("e".into()),
                StatEntry::File("f".into(), 29116),
                StatEntry::File("g".into(), 2557),
                StatEntry::File("h.lst".into(), 62596),
            ]),
            Command::Jump(Path("e".into())),
            Command::List(vec![StatEntry::File("i".into(), 584)]),
            Command::Jump(Path("..".into())),
            Command::Jump(Path("..".into())),
            Command::Jump(Path("d".into())),
            Command::List(vec![
                StatEntry::File("j".into(), 4_060_174),
                StatEntry::File("d.log".into(), 8_033_020),
                StatEntry::File("d.ext".into(), 5_626_152),
                StatEntry::File("k".into(), 7_214_296),
            ]),
        ];

        let built = input.into_iter().collect::<FileSystem>();
        let dir_sizer = DirectorySizer::for_file_system(&built);

        assert_eq!(dir_sizer.sum_under(100_000), 95437);
    }

    #[test]
    fn best_to_delete() {
        let input = vec![
            Command::Jump(Path("/".into())),
            Command::List(vec![
                StatEntry::Directory("a".into()),
                StatEntry::File("b.txt".into(), 14_848_514),
                StatEntry::File("c.dat".into(), 8_504_156),
                StatEntry::Directory("d".into()),
            ]),
            Command::Jump(Path("a".into())),
            Command::List(vec![
                StatEntry::Directory("e".into()),
                StatEntry::File("f".into(), 29116),
                StatEntry::File("g".into(), 2557),
                StatEntry::File("h.lst".into(), 62596),
            ]),
            Command::Jump(Path("e".into())),
            Command::List(vec![StatEntry::File("i".into(), 584)]),
            Command::Jump(Path("..".into())),
            Command::Jump(Path("..".into())),
            Command::Jump(Path("d".into())),
            Command::List(vec![
                StatEntry::File("j".into(), 4_060_174),
                StatEntry::File("d.log".into(), 8_033_020),
                StatEntry::File("d.ext".into(), 5_626_152),
                StatEntry::File("k".into(), 7_214_296),
            ]),
        ];

        let built = input.into_iter().collect::<FileSystem>();
        let dir_sizer = DirectorySizer::for_file_system(&built);

        assert_eq!(
            dir_sizer.smallest_to_get_under(40_000_000),
            Some((24_933_642usize, "d"))
        );
    }
}

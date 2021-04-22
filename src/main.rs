// This is the main function
use std::env;
use std::fs::File;
use std::io::{self, BufReader, BufRead, Write, Lines};
#[macro_use] extern crate lazy_static;

extern crate regex;
use regex::{RegexBuilder, Regex};

mod node;
use node::{Node, setProperty, addChild};
use std::process::Command;
use std::collections::HashMap;
use std::vec::Vec;

lazy_static!{
static ref regexString: Regex = Regex::new(r"^# (?P<name>.+)$").unwrap();
}
lazy_static!{
static ref regexSetLasy: Vec<(Regex, u16)> = {
  let mut m = Vec::new();
  m.push((Regex::new(r"^# (?P<name>.+)$").unwrap(), 1));
  m.push((Regex::new(r"^## (?P<name>.+)$").unwrap(), 2));
  m.push((Regex::new(r"^### (?P<name>Planned)").unwrap(), 3));
  m.push((Regex::new(r"^### (?P<name>Done)").unwrap(), 3));
  m.push((Regex::new(r"^### (?P<name>Dependencies)").unwrap(), 3));
  m.push((Regex::new(r"^- \\[(?P<name>.+)\\]\\((.*)\\)").unwrap(), 4));
  m.push((Regex::new(r"(^\\d+\\.|-) (?P<name>.*) (?P<cost>\\d\\+) Euro").unwrap(), 4));
  m.push((Regex::new(r"(^\\d+\\.|-) (?P<cost>\\d\\+) Euro (?P<name>.*)").unwrap(), 4));
  m.push((Regex::new(r"^\\d+\\. (?P<name>.+)").unwrap(), 4));
  m.push((Regex::new(r"^- (?P<name>.+)").unwrap(), 4));
  m
};
}
static regexSet: &'static[(&str, u16)] = &[
  ("^# (?P<name>.+)$", 1),
  ("^## (?P<name>.+)$", 2),
  ("^### (?P<name>Planned)", 3),
  ("^### (?P<name>Done)", 3),
  ("^### (?P<name>Dependencies)", 3),
  ("^- \\[(?P<name>.+)\\]\\((.*)\\)", 4),
  ("(^\\d+\\.|-) (?P<name>.*) (?P<cost>[\\d.]+) Euro", 4),
  ("(^\\d+\\.|-) (?P<name>.*) (?P<day>\\d+) Tag", 4),
  ("(^\\d+\\.|-) (?P<cost>[\\d.]+) Euro (?P<name>.*)", 4),
  ("(^\\d+\\.|-) (?P<day>\\d+) Tage (?P<name>.*)", 4),
  ("(^\\d+\\.|-) (?P<name>.*) (?P<cost>\\d+) Euro .*(?P<day>\\d+) Tag", 4),
  ("(^\\d+\\.|-) (?P<name>.*) (?P<day>\\d+) Tage* .*(?P<cost>\\d+) Euro", 4),
  ("^\\d+\\. (?P<name>.+)", 4),
  ("^- (?P<dependency>.+) \\[(?P<name>.+)\\]\\((?P<file>.+)\\)", 4),
  ("^- (?P<name>.+)", 4),
];
  

fn parseMDLine(
  rootNode:&mut Node,
  parentLine: &mut std::io::Lines<std::io::BufReader<File>>,
  parentLevel: u16,
  filePath: &std::path::Path,
  )
  -> Result<std::string::String, io::Error> {
  let mut lineItr = parentLine.next();
  let mut done = !lineItr.is_some();
  let mut linePtr  = lineItr.unwrap_or(Ok("".to_string()));
  let mut line = linePtr.unwrap();
  while !done{
    println !("parse\n{}\nwith parentLevel {}", line, parentLevel);
    let mut regexIter = regexSet.into_iter();
    //for test in regexSetLasy.into_iter(){
    loop{
      match regexIter.next(){
        Some((regexStr, level)) =>{
          let regex = RegexBuilder::new(regexStr).build().unwrap();
          let lineClone = line.clone();
          match regex.captures(&lineClone) {
            Some(caps) =>{
              match caps.name("name") {
                Some(nodeName) =>{
                  println!("found work package: {:?}", &caps["name"]);
                  let currLevel = *level;
                  println !("parsed\n{}\ncurr level {}", regex.as_str(), currLevel);
                  if currLevel <= parentLevel{
                    println !("parse again\n{}", line);
                    return Ok(line.clone());
                  //} else if currLevel == parentLevel+1{
                  } else {
                    // create node
                    let file = caps.name("file").map_or(filePath, |m| &std::path::Path::new(m.as_str()));
                    let mut newNode = if currLevel > 2 {
                      Node::new(nodeName.as_str().to_string(), file)
                    } else {
                      Node::new2(nodeName.as_str().to_string(), file, node::VARIANT::Package)
                    };
                    // cost
                    println !("parsed cost {}", caps.name("cost").map_or("0" , |m| m.as_str()).parse::<f32>().unwrap_or(0.0));
                    let cost = caps.name("cost").map_or("0", |m| m.as_str()).parse::<f32>().unwrap_or(0.0);
                    newNode.cost= cost;
                    let day = caps.name("day").map_or("0", |m| m.as_str()).parse::<u32>().unwrap_or(0);
                    newNode.day = day;
                    // dependencies
                    let dependency = caps.name("dependency").map_or("Unknown", |m| m.as_str());
                    setProperty(&mut newNode, dependency);
                    // parse children
                    line = parseMDLine(&mut newNode, parentLine, currLevel, filePath).unwrap().clone();
                    addChild(rootNode, newNode);
                    break;
                  //} else {
                  //  // todo error
                  //  return Ok(line.clone());
                  }
                },
                None => {
                }
              }
            },
            None => {} 
          }
        }
        None => {
          lineItr = parentLine.next();
          if lineItr.is_some() {
            println !("cont.");
            linePtr  = lineItr.unwrap();
            line = linePtr.unwrap();
          } else {
            done = true;
          }
            break;
          }
      }
    };
  };
  Ok("".to_string())
}

fn parseMDFile(rootNode:&mut Node, path: &std::path::Path) -> Result<(), io::Error> {
  // check if valid path
  match path.extension().unwrap().to_str(){
    Some("md") => {
      println !("parse file {}", path.display());
      let buffered = BufReader::new(File::open(path)?);
      let nodeName = path.file_name().unwrap().to_str().unwrap().to_string();
      let mut newNode = Node::new2(nodeName, path, node::VARIANT::Group);
      //setOwnership(rootNode, &newNode);
      parseMDLine(&mut newNode, &mut buffered.lines(), 0, path);
      addChild(rootNode, newNode);
    },
    _ => {
      return Err(std::io::Error::new(std::io::ErrorKind::Other, "only md file are to parse"));
    }
  }
  Ok(())
}

fn parseDir(rootNode:&mut Node, dir: &std::path::Path) -> io::Result<()>{
  for entry in dir.read_dir().expect("read_dir call failed") {
    match entry {
      Ok(entry) => {
        let newPath=entry.path();
        if newPath.is_dir() {
          let nodeName = newPath.file_name().unwrap().to_str().unwrap().to_string();
          let mut newNode = Node::new2(nodeName, dir, node::VARIANT::Group);
          parseDir(&mut newNode, &newPath);
          addChild(rootNode, newNode);
        }else{
          parseMDFile(rootNode, &newPath);
        }
      },
      Err(e) => {
        return Err(e);
      }
    }
  }
  Ok(())
}

//fn handleInput(rootNode:&mut Node, input: std::string::String) -> io::Result<()>{
fn handleInput(rootNode:&mut Node, input: &str) -> io::Result<()>{
  let path = std::path::Path::new(input);
  if path.is_dir() {
    parseDir(rootNode, path)
  }else{
    parseMDFile(rootNode, path)
  }
}

fn getDotShape(kind: &node::VARIANT) -> &'static str{
  match kind{
    node::VARIANT::Root => "house",
    node::VARIANT::Group => "folder",
    node::VARIANT::Package => "hexagon",
    node::VARIANT::Item => "ellipse",
    _ => "rect",
  }
}

fn getDotColor(kind: &node::STATE) -> &'static str{
  match kind{
    node::STATE::Open => "lightBlue",
    node::STATE::Planed => "blue",
    node::STATE::Working => "yellow",
    node::STATE::Closed => "green",
    _ => "grey",
  }
}

fn getDotStyle(kind: &node::PROPERTY) -> &'static str{
  match kind{
    node::PROPERTY::Muss => "bold",
    node::PROPERTY::Soll => "solid",
    node::PROPERTY::NiceToHave => "dashed",
    _ => "dotted",
  }
}

fn getDotFileName(file: &std::string::String) -> std::string::String{
  let path = std::path::Path::new(&file);
  let stripedPath = path.strip_prefix("../");
  if stripedPath.is_ok(){
    getDotFileName(&stripedPath.unwrap().to_str().unwrap().to_string())
  } else {
    match path.extension(){
        None => format!("{}/", file),
        Some(os_str) => {
          match os_str.to_str() {
            Some("md") => os_str.to_str().unwrap().to_string(),
            _ => panic!("only md file is supported now")
          }
        }
    }
  }
}

fn generateDot( rootNode:& Node, file: &mut std::fs::File) -> Result<(), io::Error> {
  writeln!(file, "node_{} [label=\"{}\n{} Euro {} Days\" shape=\"{}\" color=black fillcolor=\"{}\" style=\"filled, {}\" URL=\"index.html#!{}\"]", rootNode.id, rootNode.name, rootNode.cost,rootNode.day, getDotShape(&rootNode.kind), getDotColor(&rootNode.state) , getDotStyle(&rootNode.property), getDotFileName(&rootNode.file));
      for child in &rootNode.children
      {
          match (&child.kind, &child.state){
            (node::VARIANT::Package, node::STATE::Unknown) => {;}
            (node::VARIANT::Item, node::STATE::Unknown) => {;}
            (node::VARIANT::State, node::STATE::Closed) => {;}
            (node::VARIANT::Item, node::STATE::Closed) => {;}
            (_, _) => {
            writeln!(file, "node_{} -> node_{}", rootNode.id, child.id);
            generateDot(&child, file) ;
            }
    }
  }
  Ok(())
}

fn handleOutput(rootNode:& Node, output: &str) -> io::Result<()>{
  let pathName = if output.starts_with("/") {
      output.to_string()
  } else {
      format!("./{}", output)
  };
  let path = std::path::Path::new(&pathName);
  let (filename, dir) = if path.is_dir(){
    ("output", path)  
  } else {
      match path.extension().unwrap().to_str(){
        Some("md") => (path.file_stem().unwrap().to_str().unwrap(),path.parent().unwrap()),
        _ => (path.file_name().unwrap().to_str().unwrap(), path.parent().unwrap())
      }
  };
  assert_eq!(std::path::Path::new(".").exists(), true);
  println!("path dir is {}", dir.display());
  if !dir.exists(){
      std::fs::create_dir_all(dir)?;
  }
  let outputDir = dir.to_str().unwrap();
  let outputDotFilename = format!("{}/{}.dot", outputDir, filename);
  let mut outputFile = File::create(&outputDotFilename)?;
  //let mut outputWriter = LineWriter::new(outputFile);
  writeln!(outputFile, "digraph status {{");
  writeln!(outputFile, "ranksep=.75");
  writeln!(outputFile, "rankdir=LR");
  writeln!(outputFile, "overlap=prism; overlap_scaling=0.01; radio=\"compress\"");
  writeln!(outputFile, "graph [fontsize = 12]");
  writeln!(outputFile, "node [fontsize = 10]");
  writeln!(outputFile, "edge [fontsize = 8]");
  generateDot(&rootNode, &mut outputFile);
  writeln!(outputFile, "}}")?;
  outputFile.flush()?;

  let outputMDFilename = format!("{}/{}.md", outputDir, filename);
  let outputPngFilename = format!("{}/{}.png", outputDir, filename);
  let outputHTMLFilename = format!("{}/{}.html", outputDir, filename);
  let mut outputMDFile = File::create(&outputMDFilename)?;
  writeln!(outputMDFile, "<div>");
  //if cfg!(target_os = "windows") {  
    // TODO not supported
  //} else { 
  Command::new("dot")
      .arg("-Tcmapx")
      .arg(&outputDotFilename)
      .arg("-o")
      .arg(&outputHTMLFilename)
      .arg("-Tpng")
      .arg("-o")
      .arg(&outputPngFilename)
      .output().expect("failed");
  println!("Command:\ndot -Tcmapx {} -o {} -Tpng -o {}",outputDotFilename, outputHTMLFilename, outputPngFilename);
  let contents = std::fs::read_to_string(outputHTMLFilename).expect("Something went wrong reading the file");
  writeln!(outputMDFile, "{}", contents.replace("\n", ""));

  writeln!(outputMDFile, "<img src=\"{}.png\" usemap=\"#status\" />", filename);
  writeln!(outputMDFile, "</div>");
  outputMDFile.flush()?;
  Ok(())

}

fn main() -> io::Result<()> {
  let args : Vec<String> = env::args().skip(1).collect();

  let mut rootNode = Node::new2("Haus Renovierung".to_string(), std::path::Path::new("."), node::VARIANT::Root);

  let srcDir = args.get(0).map_or("../../template", |v| &v);
  handleInput(&mut rootNode, srcDir.clone())?;

  handleOutput(&rootNode, args.get(1).map_or("temp", |v| &v))?;
  Ok(())
}

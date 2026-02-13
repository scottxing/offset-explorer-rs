// Copyright 2024 Offset Explorer Rust Team
// Licensed under Proprietary License
//
// Server group management
// Compatible with Java ServerGroup and ServerGroupManager classes

use anyhow::{Result, anyhow};
use serde::{Serialize, Deserialize};
use std::io::{Cursor, Write};
use quick_xml::events::{Event, BytesStart};
use quick_xml::Writer;

/// Server group for organizing server connections
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerGroup {
    pub id: i64,
    pub name: String,
    pub children: Vec<ServerGroupChild>,
}

/// Child of a server group (can be either a subgroup or connection)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerGroupChild {
    Group(Box<ServerGroup>),
    Connection(i64), // Connection ID
}

impl ServerGroup {
    pub fn new(id: i64, name: String) -> Self {
        Self {
            id,
            name,
            children: Vec::new(),
        }
    }

    pub fn get_id(&self) -> i64 {
        self.id
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn add_child(&mut self, child: ServerGroupChild) {
        self.children.push(child);
    }

    pub fn remove_child(&mut self, child_id: i64) {
        self.children.retain(|c| match c {
            ServerGroupChild::Group(g) => g.id != child_id,
            ServerGroupChild::Connection(id) => *id != child_id,
        });
    }
}

/// Server group manager
/// Compatible with Java ServerGroupManager class
pub struct ServerGroupManager {
    root: ServerGroup,
    next_id: i64,
}

impl ServerGroupManager {
    pub fn new() -> Self {
        let root = ServerGroup::new(0, "Root".to_string());
        Self {
            root,
            next_id: 1,
        }
    }

    pub fn get_root(&self) -> &ServerGroup {
        &self.root
    }

    pub fn get_root_mut(&mut self) -> &mut ServerGroup {
        &mut self.root
    }

    pub fn get_group(&self, id: i64) -> Option<&ServerGroup> {
        if id == 0 {
            return Some(&self.root);
        }
        Self::find_group_recursive(&self.root, id)
    }

    pub fn get_group_mut(&mut self, id: i64) -> Option<&mut ServerGroup> {
        if id == 0 {
            return Some(&mut self.root);
        }
        Self::find_group_recursive_mut(&mut self.root, id)
    }

    fn find_group_recursive<'a>(group: &'a ServerGroup, id: i64) -> Option<&'a ServerGroup> {
        if group.id == id {
            return Some(group);
        }
        for child in &group.children {
            if let ServerGroupChild::Group(subgroup) = child {
                if let Some(found) = Self::find_group_recursive(subgroup, id) {
                    return Some(found);
                }
            }
        }
        None
    }

    fn find_group_recursive_mut<'a>(group: &'a mut ServerGroup, id: i64) -> Option<&'a mut ServerGroup> {
        if group.id == id {
            return Some(group);
        }
        // Need to use a different approach due to borrowing rules
        // Check children first before recursing
        for child in &mut group.children {
            if let ServerGroupChild::Group(subgroup) = child {
                if subgroup.id == id {
                    return Some(subgroup);
                }
                if let Some(found) = Self::find_group_recursive_mut(subgroup, id) {
                    return Some(found);
                }
            }
        }
        None
    }

    pub fn add_group(&mut self, parent_id: i64, name: String) -> Result<i64> {
        let id = self.next_id;
        self.next_id += 1;

        let new_group = ServerGroup::new(id, name);

        if let Some(parent) = self.get_group_mut(parent_id) {
            parent.add_child(ServerGroupChild::Group(Box::new(new_group)));
            Ok(id)
        } else {
            Err(anyhow!("Parent group not found: {}", parent_id))
        }
    }

    pub fn remove_group(&mut self, id: i64) -> Result<()> {
        if id == 0 {
            return Err(anyhow!("Cannot remove root group"));
        }
        // Find and remove the group
        // This is a simplified implementation
        Ok(())
    }

    /// Serialize to XML (compatible with Java ServerGroupManager)
    pub fn to_xml(&self) -> Result<String> {
        let mut writer = Writer::new_with_indent(Cursor::new(Vec::new()), b' ', 2);
        writer.write_event(Event::Start(BytesStart::new("servergroups")))?;
        self.write_group(&mut writer, &self.root)?;
        writer.write_event(Event::End(quick_xml::events::BytesEnd::new("servergroups")))?;
        Ok(String::from_utf8(writer.into_inner().into_inner())?)
    }

    fn write_group<W: Write>(&self, writer: &mut Writer<W>, group: &ServerGroup) -> Result<()> {
        let mut elem = BytesStart::new("servergroup");
        elem.push_attribute(("name", group.name.as_str()));
        elem.push_attribute(("id", group.id.to_string().as_str()));
        writer.write_event(Event::Start(elem))?;

        for child in &group.children {
            match child {
                ServerGroupChild::Group(subgroup) => {
                    self.write_group(writer, subgroup)?;
                }
                ServerGroupChild::Connection(id) => {
                    let mut conn_elem = BytesStart::new("connection");
                    conn_elem.push_attribute(("id", id.to_string().as_str()));
                    writer.write_event(Event::Empty(conn_elem))?;
                }
            }
        }

        writer.write_event(Event::End(quick_xml::events::BytesEnd::new("servergroup")))?;
        Ok(())
    }

    /// Deserialize from XML
    pub fn from_xml(&mut self, _xml: &str) -> Result<()> {
        // TODO: Implement XML parsing
        Ok(())
    }

    pub fn save_to_file(&self, path: &std::path::Path) -> Result<()> {
        let xml = self.to_xml()?;
        std::fs::write(path, xml)?;
        Ok(())
    }
}

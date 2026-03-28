use crate::bean::{Bean, BeanStatus, BeanType};
use crate::render;

/// Type sections in display order.
const TYPE_SECTIONS: &[(BeanType, &str)] = &[
    (BeanType::Epic, "Epics"),
    (BeanType::Feature, "Features"),
    (BeanType::Task, "Tasks"),
    (BeanType::Bug, "Bugs"),
    (BeanType::Spike, "Spikes"),
    (BeanType::Chore, "Chores"),
];

/// Render the All Tasks chapter as a single page with all bean content inline.
pub fn render(beans: &[Bean]) -> String {
    let mut content = String::from("# All Tasks\n\n");

    // Collect epic-to-children mapping
    let epic_children = |epic_id: &str| -> Vec<&Bean> {
        beans
            .iter()
            .filter(|b| b.frontmatter.parent.as_deref() == Some(epic_id))
            .collect()
    };

    // Table of contents
    for (bean_type, section_label) in TYPE_SECTIONS {
        let matching: Vec<&Bean> = beans
            .iter()
            .filter(|b| &b.frontmatter.bean_type == bean_type)
            .collect();

        if matching.is_empty() {
            continue;
        }

        content.push_str(&format!("## {section_label}\n\n"));

        for bean in &matching {
            content.push_str(&format!(
                "- [{}](#{}) — *{}*",
                bean.frontmatter.title,
                bean.id,
                render::status_label(&bean.frontmatter.status)
            ));
            content.push('\n');

            // For epics, also list subtasks
            if *bean_type == BeanType::Epic {
                let children = epic_children(&bean.id);
                for child in &children {
                    content.push_str(&format!(
                        "  - [{}](#{}) — *{}*\n",
                        child.frontmatter.title,
                        child.id,
                        render::status_label(&child.frontmatter.status)
                    ));
                }
            }
        }

        content.push('\n');
    }

    // Draft beans in TOC
    let drafts: Vec<&Bean> = beans
        .iter()
        .filter(|b| b.frontmatter.status == BeanStatus::Draft)
        .collect();

    if !drafts.is_empty() {
        content.push_str("## Drafts\n\n");
        for bean in &drafts {
            content.push_str(&format!(
                "- [{}](#{}) — *Draft*\n",
                bean.frontmatter.title, bean.id
            ));
        }
        content.push('\n');
    }

    // Separator
    content.push_str("---\n\n");

    // Full bean details inline
    for bean in beans {
        content.push_str(&render::render_bean_section(bean, beans));
        content.push_str("\n---\n\n");
    }

    content
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bean::BeanFrontmatter;

    fn make_bean(
        id: &str,
        title: &str,
        status: BeanStatus,
        bean_type: BeanType,
        parent: Option<&str>,
    ) -> Bean {
        Bean {
            id: id.to_string(),
            frontmatter: BeanFrontmatter {
                title: title.to_string(),
                status,
                bean_type,
                priority: "normal".to_string(),
                tags: vec![],
                parent: parent.map(|s| s.to_string()),
                blocked_by: vec![],
            },
            body: format!("Body of {title}"),
        }
    }

    #[test]
    fn tasks_groups_by_type() {
        let beans = vec![
            make_bean("b-1", "A feature", BeanStatus::Todo, BeanType::Feature, None),
            make_bean("b-2", "A bug", BeanStatus::Todo, BeanType::Bug, None),
            make_bean("b-3", "A task", BeanStatus::Todo, BeanType::Task, None),
        ];
        let content = render(&beans);
        assert!(content.contains("## Features"));
        assert!(content.contains("## Tasks"));
        assert!(content.contains("## Bugs"));
    }

    #[test]
    fn tasks_has_anchor_links() {
        let beans = vec![
            make_bean("b-1", "Task one", BeanStatus::Todo, BeanType::Task, None),
        ];
        let content = render(&beans);
        assert!(content.contains("(#b-1)"));
    }

    #[test]
    fn tasks_epic_lists_subtasks_inline() {
        let beans = vec![
            make_bean("b-epic", "My Epic", BeanStatus::InProgress, BeanType::Epic, None),
            make_bean("b-sub1", "Sub 1", BeanStatus::Done, BeanType::Task, Some("b-epic")),
            make_bean("b-sub2", "Sub 2", BeanStatus::Todo, BeanType::Feature, Some("b-epic")),
        ];
        let content = render(&beans);
        assert!(content.contains("Sub 1"));
        assert!(content.contains("Sub 2"));
    }

    #[test]
    fn tasks_drafts_appear_at_end() {
        let beans = vec![
            make_bean("b-1", "Active", BeanStatus::Todo, BeanType::Task, None),
            make_bean("b-2", "Draft one", BeanStatus::Draft, BeanType::Task, None),
        ];
        let content = render(&beans);
        let tasks_pos = content.find("## Tasks").unwrap();
        let drafts_pos = content.find("## Drafts").unwrap();
        assert!(tasks_pos < drafts_pos);
    }

    #[test]
    fn tasks_renders_full_bean_content() {
        let beans = vec![
            make_bean("b-1", "Task one", BeanStatus::Todo, BeanType::Task, None),
        ];
        let content = render(&beans);
        assert!(content.contains("Body of Task one"));
    }
}

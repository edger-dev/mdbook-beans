use mdbook_preprocessor::book::{BookItem, Chapter};

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

/// Render a type section as a sub-chapter containing all beans of that type inline.
fn render_type_section(label: &str, beans: &[&Bean], all_beans: &[Bean]) -> Chapter {
    let mut content = format!("# {label}\n\n");

    for bean in beans {
        content.push_str(&render::render_bean_section(bean, all_beans));
        content.push_str("\n---\n\n");
    }

    let slug = label.to_lowercase();
    let path = format!("beans/{slug}.md");
    let mut chapter = Chapter::new(label, content, &path, vec![]);
    chapter.source_path = None;
    chapter
}

/// Render the All Tasks chapter with type sections as collapsible sub-items.
/// Returns (index page content, sub_item chapters per type section).
pub fn render(beans: &[Bean]) -> (String, Vec<BookItem>) {
    let mut content = String::from("# All Tasks\n\n");
    let mut sub_items: Vec<BookItem> = Vec::new();

    let epic_children = |epic_id: &str| -> Vec<&Bean> {
        beans
            .iter()
            .filter(|b| b.frontmatter.parent.as_deref() == Some(epic_id))
            .collect()
    };

    for (bean_type, section_label) in TYPE_SECTIONS {
        let matching: Vec<&Bean> = beans
            .iter()
            .filter(|b| &b.frontmatter.bean_type == bean_type)
            .collect();

        if matching.is_empty() {
            continue;
        }

        // Index page: list beans per type
        content.push_str(&format!("## {section_label}\n\n"));

        for bean in &matching {
            content.push_str(&format!(
                "- {} — *{}*",
                bean.frontmatter.title,
                render::status_label(&bean.frontmatter.status)
            ));
            content.push('\n');

            if *bean_type == BeanType::Epic {
                let children = epic_children(&bean.id);
                for child in &children {
                    content.push_str(&format!(
                        "  - {} — *{}*\n",
                        child.frontmatter.title,
                        render::status_label(&child.frontmatter.status)
                    ));
                }
            }
        }

        content.push('\n');

        // Sub-chapter for this type
        sub_items.push(BookItem::Chapter(render_type_section(
            section_label,
            &matching,
            beans,
        )));
    }

    // Draft beans
    let drafts: Vec<&Bean> = beans
        .iter()
        .filter(|b| b.frontmatter.status == BeanStatus::Draft)
        .collect();

    if !drafts.is_empty() {
        content.push_str("## Drafts\n\n");
        for bean in &drafts {
            content.push_str(&format!(
                "- {} — *Draft*\n",
                bean.frontmatter.title
            ));
        }
        content.push('\n');

        sub_items.push(BookItem::Chapter(render_type_section(
            "Drafts", &drafts, beans,
        )));
    }

    (content, sub_items)
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
        let (content, _) = render(&beans);
        assert!(content.contains("## Features"));
        assert!(content.contains("## Tasks"));
        assert!(content.contains("## Bugs"));
    }

    #[test]
    fn tasks_creates_type_sub_items() {
        let beans = vec![
            make_bean("b-1", "A feature", BeanStatus::Todo, BeanType::Feature, None),
            make_bean("b-2", "A bug", BeanStatus::Todo, BeanType::Bug, None),
            make_bean("b-3", "A task", BeanStatus::Todo, BeanType::Task, None),
        ];
        let (_, sub_items) = render(&beans);
        assert_eq!(sub_items.len(), 3); // Features, Tasks, Bugs

        let names: Vec<&str> = sub_items
            .iter()
            .filter_map(|item| {
                if let BookItem::Chapter(ch) = item {
                    Some(ch.name.as_str())
                } else {
                    None
                }
            })
            .collect();
        assert!(names.contains(&"Features"));
        assert!(names.contains(&"Tasks"));
        assert!(names.contains(&"Bugs"));
    }

    #[test]
    fn tasks_epic_lists_subtasks_inline() {
        let beans = vec![
            make_bean("b-epic", "My Epic", BeanStatus::InProgress, BeanType::Epic, None),
            make_bean("b-sub1", "Sub 1", BeanStatus::Done, BeanType::Task, Some("b-epic")),
            make_bean("b-sub2", "Sub 2", BeanStatus::Todo, BeanType::Feature, Some("b-epic")),
        ];
        let (content, _) = render(&beans);
        assert!(content.contains("Sub 1"));
        assert!(content.contains("Sub 2"));
    }

    #[test]
    fn tasks_drafts_appear_at_end() {
        let beans = vec![
            make_bean("b-1", "Active", BeanStatus::Todo, BeanType::Task, None),
            make_bean("b-2", "Draft one", BeanStatus::Draft, BeanType::Task, None),
        ];
        let (content, sub_items) = render(&beans);
        let tasks_pos = content.find("## Tasks").unwrap();
        let drafts_pos = content.find("## Drafts").unwrap();
        assert!(tasks_pos < drafts_pos);

        // Drafts should be a sub-item too
        let last = sub_items.last().unwrap();
        if let BookItem::Chapter(ch) = last {
            assert_eq!(ch.name, "Drafts");
        }
    }

    #[test]
    fn type_section_contains_bean_content() {
        let beans = vec![
            make_bean("b-1", "Task one", BeanStatus::Todo, BeanType::Task, None),
        ];
        let (_, sub_items) = render(&beans);
        if let BookItem::Chapter(ch) = &sub_items[0] {
            assert!(ch.content.contains("Body of Task one"));
        }
    }
}

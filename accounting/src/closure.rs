use crate::id::AccountId;
use std::collections::HashMap;

/// 用于闭包计算的账户节点（简化视图）
#[derive(Debug, Clone)]
pub struct AccountNode {
    pub id: AccountId,
    pub parent_id: Option<AccountId>,
    pub full_name: String,
}

/// 计算闭包表
///
/// 返回每个账户到其后代列表（含自身）的映射
pub fn compute_closure(accounts: &[AccountNode]) -> HashMap<AccountId, Vec<AccountId>> {
    let mut children: HashMap<AccountId, Vec<AccountId>> = HashMap::new();
    for acc in accounts {
        if let Some(parent) = acc.parent_id {
            children.entry(parent).or_default().push(acc.id);
        }
    }

    fn collect_descendants(
        id: AccountId,
        children: &HashMap<AccountId, Vec<AccountId>>,
        result: &mut Vec<AccountId>,
    ) {
        if let Some(kids) = children.get(&id) {
            for &child in kids {
                result.push(child);
                collect_descendants(child, children, result);
            }
        }
    }

    let mut closure: HashMap<AccountId, Vec<AccountId>> = HashMap::new();
    for acc in accounts {
        let mut descendants = vec![acc.id];
        collect_descendants(acc.id, &children, &mut descendants);
        closure.insert(acc.id, descendants);
    }

    closure
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::id::AccountId;

    #[test]
    fn test_root_account() {
        let accounts = vec![AccountNode {
            id: AccountId(1),
            parent_id: None,
            full_name: "Assets".to_string(),
        }];
        let closure = compute_closure(&accounts);
        assert_eq!(closure.get(&AccountId(1)).unwrap(), &vec![AccountId(1)]);
    }

    #[test]
    fn test_parent_child() {
        let accounts = vec![
            AccountNode {
                id: AccountId(1),
                parent_id: None,
                full_name: "Assets".to_string(),
            },
            AccountNode {
                id: AccountId(2),
                parent_id: Some(AccountId(1)),
                full_name: "Assets:Cash".to_string(),
            },
        ];
        let closure = compute_closure(&accounts);
        assert_eq!(
            closure.get(&AccountId(1)).unwrap(),
            &vec![AccountId(1), AccountId(2)]
        );
        assert_eq!(closure.get(&AccountId(2)).unwrap(), &vec![AccountId(2)]);
    }

    #[test]
    fn test_deep_hierarchy() {
        let accounts = vec![
            AccountNode {
                id: AccountId(1),
                parent_id: None,
                full_name: "Assets".to_string(),
            },
            AccountNode {
                id: AccountId(2),
                parent_id: Some(AccountId(1)),
                full_name: "Assets:Bank".to_string(),
            },
            AccountNode {
                id: AccountId(3),
                parent_id: Some(AccountId(2)),
                full_name: "Assets:Bank:Checking".to_string(),
            },
        ];
        let closure = compute_closure(&accounts);
        let root = closure.get(&AccountId(1)).unwrap();
        assert!(root.contains(&AccountId(1)));
        assert!(root.contains(&AccountId(2)));
        assert!(root.contains(&AccountId(3)));
    }
}

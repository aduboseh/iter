#!/usr/bin/env python3
"""Fix test files for Phase 3 - update ID handling from UUID to u64."""

import re
import glob

def fix_file(filepath):
    with open(filepath, 'r', encoding='utf-8') as f:
        content = f.read()
    
    original = content
    
    # Replace UUID placeholders with numeric IDs
    content = content.replace('"00000000-0000-0000-0000-000000000000"', '"999999"')
    
    # Replace patterns like:
    # let content = create_resp.get_content_text().unwrap();
    # let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
    # let node_id = parsed["id"].as_str().unwrap();
    # With:
    # let node_id = extract_node_id(&create_resp);
    
    pattern1 = r'let content = ([a-z_]+)\.get_content_text\(\)\.unwrap\(\);\s*let parsed: serde_json::Value = serde_json::from_str\(&content\)\.unwrap\(\);\s*let ([a-z_0-9]+) = parsed\["id"\]\.as_str\(\)\.unwrap\(\);'
    content = re.sub(pattern1, r'let \2 = extract_node_id(&\1);', content)
    
    # Replace pattern:
    # let node1: serde_json::Value = serde_json::from_str(&node1_resp.get_content_text().unwrap()).unwrap();
    # let node1_id = node1["id"].as_str().unwrap();
    pattern2 = r'let ([a-z_0-9]+): serde_json::Value = serde_json::from_str\(&([a-z_0-9]+)\.get_content_text\(\)\.unwrap\(\)\)\.unwrap\(\);\s*let ([a-z_0-9]+) = \1\["id"\]\.as_str\(\)\.unwrap\(\);'
    content = re.sub(pattern2, r'let \3 = extract_node_id(&\2);', content)
    
    # Fix queried["id"].as_str().unwrap() comparisons - need to handle differently
    # These need to compare as u64 or string
    pattern3 = r'assert_eq!\(queried\["id"\]\.as_str\(\)\.unwrap\(\), node_id\);'
    content = re.sub(pattern3, r'assert_eq!(queried["id"].as_u64().unwrap().to_string(), node_id);', content)
    
    if content != original:
        with open(filepath, 'w', encoding='utf-8') as f:
            f.write(content)
        print(f'Updated: {filepath}')
        return True
    return False

def main():
    updated = 0
    for filepath in glob.glob('tests/**/*.rs', recursive=True):
        if fix_file(filepath):
            updated += 1
    print(f'\nTotal files updated: {updated}')

if __name__ == '__main__':
    main()

-- Insert default comments component template
INSERT INTO component_templates (name, component_type, template_data, breakpoints, is_default, is_active)
VALUES (
    'Default Comments Section', 
    'Comments',
    '{"enabled": true, "per_page": 20, "avatar_size": 48, "show_auth_prompt": true, "moderation": false}'::jsonb,
    '{"mobile": {"max_width": "767px"}, "tablet": {"max_width": "1024px"}, "desktop": {"min_width": "1025px"}}'::jsonb,
    true,
    true
);
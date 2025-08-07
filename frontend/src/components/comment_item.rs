use yew::prelude::*;
use crate::services::api_service::CommentWithGravatar;

#[derive(Properties, PartialEq)]
pub struct CommentItemProps {
    pub comment: CommentWithGravatar,
}

#[function_component(CommentItem)]
pub fn comment_item(props: &CommentItemProps) -> Html {
    let comment = &props.comment;
    let author_name = comment.author_username.as_ref()
        .map(|s| s.as_str())
        .unwrap_or("Anonymous");
    
    // Format the timestamp
    let formatted_time = comment.created_at.as_ref()
        .and_then(|time_str| {
            chrono::NaiveDateTime::parse_from_str(time_str, "%Y-%m-%d %H:%M:%S")
                .ok()
                .map(|dt| dt.format("%b %d, %Y at %I:%M %p").to_string())
        })
        .unwrap_or_else(|| "Just now".to_string());

    html! {
        <div class="comment-bubble">
            <div class="comment-avatar">
                <img 
                    src={comment.gravatar_url.clone()} 
                    alt={format!("{}'s avatar", author_name)}
                    class="avatar-image"
                />
            </div>
            <div class="comment-content">
                <div class="comment-header">
                    <span class="comment-author">{author_name}</span>
                    <span class="comment-time">{formatted_time}</span>
                </div>
                <div class="comment-text">
                    {comment.content.clone()}
                </div>
            </div>
        </div>
    }
}
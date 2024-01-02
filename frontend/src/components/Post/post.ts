export interface UserPostEntry {
    id: string;
    author: UserPostEntryAuthor;
    post: UserPostDetail;
    created_at: string;
    privacy: string;
}

export interface UserPostDetail {
    text: string;
    pictures_url: string[];
    reactions: string[];
}

export interface UserPostEntryAuthor {
    id: string;
    username: string;
}
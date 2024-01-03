export type Privacy = 'public' | 'unlisted' | 'follower' | 'private'

export interface UserPostEntry {
    id: string
    author: UserPostEntryAuthor
    post: UserPostDetail
    created_at: string
    privacy: Privacy
}

export interface UserPostDetail {
    text: string
    pictures_url: string[]
    reactions: string[]
}

export interface UserPostEntryAuthor {
    id: string
    username: string
}

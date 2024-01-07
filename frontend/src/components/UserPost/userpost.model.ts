export type Privacy = 'public' | 'unlisted' | 'follower' | 'private'

export interface UserPostEntry {
    id: string
    author: UserPostEntryAuthor
    content: string | null
    created_at: string
    privacy: Privacy
    reply_to?: string | UserPostEntry
    repost_of?: string | UserPostEntry
    repost_count: number
    reply_count: number
    quote_count: number
    favorite_count: number
    reactions: Record<string, number>
}

export interface UserPostDetail {
    text: string
    pictures_url: string[]
    reactions: string[]
}

export interface UserPostEntryAuthor {
    id: string
    username: string
    host: string
}

export interface TimelineResponse {
    posts: UserPostEntry[]
}

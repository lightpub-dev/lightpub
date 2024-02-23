export const PRIVACY_PUBLIC = 'public' as const
export const PRIVACY_UNLISTED = 'unlisted' as const
export const PRIVACY_FOLLOWERS = 'follower' as const
export const PRIVACY_PRIVATE = 'private' as const

export type Privacy =
    | typeof PRIVACY_PUBLIC
    | typeof PRIVACY_UNLISTED
    | typeof PRIVACY_FOLLOWERS
    | typeof PRIVACY_PRIVATE

export interface UserPostEntry {
    id: string
    author: UserPostEntryAuthor
    content: string | null
    created_at: string
    privacy: Privacy
    reply_to_id?: string
    reply_to?: UserPostEntry
    repost_of_id?: string
    repost_of?: UserPostEntry
    repost_count: number
    reply_count: number
    quote_count: number
    favorite_count: number
    reactions: Record<string, number>
    reposted_by_me?: string
    favorited_by_me?: boolean
    bookmarked_by_me?: boolean
    attached_files: {
        id: string
        url: string
    }[]
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
    nickname: string
    avatar: string
}

export interface TimelineResponse {
    posts: UserPostEntry[]
}

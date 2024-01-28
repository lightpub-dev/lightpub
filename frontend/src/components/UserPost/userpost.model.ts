export const PRIVACY_PUBLIC = 0 as const
export const PRIVACY_UNLISTED = 1 as const
export const PRIVACY_FOLLOWERS = 2 as const
export const PRIVACY_PRIVATE = 3 as const

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
    reactions: Record<string, {
        count: number,
        reacted_by_me?: boolean
    }>
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
    results: UserPostEntry[]
}

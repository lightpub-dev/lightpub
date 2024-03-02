import mitt from 'mitt'

type Event = {
    'post-created': void
    'create-reply': string
    'repost-created': void
    "post-deleted": void,
    "reaction-created": void,
    'reaction-deleted': void,
}

export const eventBus = mitt<Event>()

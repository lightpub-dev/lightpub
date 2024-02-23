import mitt from 'mitt'

type Event = {
    'post-created': void
    'create-reply': string
    'repost-created': void
}

export const eventBus = mitt<Event>()

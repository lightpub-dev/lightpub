import mitt from 'mitt'

type Event = {
    'post-created': void
    'create-reply': string
}

export const eventBus = mitt<Event>()

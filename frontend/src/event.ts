import mitt from 'mitt'

type Events = {
  newPostCreated: () => void
}

export const emitter = mitt<Events>()

import type { Meta, StoryObj } from '@storybook/vue3'

import UserPost from './UserPost.vue'

const meta: Meta<typeof UserPost> = {
  component: UserPost
}

export default meta
type Story = StoryObj<typeof UserPost>

/*
 *👇 Render functions are a framework specific feature to allow you control on how the component renders.
 * See https://storybook.js.org/docs/api/csf
 * to learn how to use render functions.
 */
export const Primary: Story = {
  render: (args) => ({
    components: { UserPost },
    setup() {
      return {
        args
      }
    },
    template:
      '<UserPost :nickname="args.nickname" :username="args.username" :host="args.host" :content="args.content" :createdAt="args.createdAt" />'
  }),
  args: {
    nickname: 'nick',
    username: 'user',
    host: 'example.com',
    content: 'hi content',
    createdAt: new Date()
  }
}

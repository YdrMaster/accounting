import { mount } from '@vue/test-utils'
import { describe, expect, it } from 'vitest'
import HelloWorld from '../HelloWorld.vue'

describe('HelloWorld', () => {
  it('renders the passed message', () => {
    const wrapper = mount(HelloWorld, {
      props: { msg: 'Hello Vitest' },
    })
    expect(wrapper.text()).toContain('Hello Vitest')
  })

  it('increments count when button is clicked', async () => {
    const wrapper = mount(HelloWorld, {
      props: { msg: 'Vue 3.6 SPA' },
    })
    const button = wrapper.find('button')
    await button.trigger('click')
    expect(button.text()).toContain('count is 1')
  })
})

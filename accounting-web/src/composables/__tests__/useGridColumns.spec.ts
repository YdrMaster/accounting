import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import { defineComponent, ref } from 'vue'
import { useGridColumns } from '../useGridColumns'

describe('useGridColumns', () => {
  it('reads --grid-columns from the element', async () => {
    const TestComp = defineComponent({
      setup() {
        const el = ref<HTMLElement>()
        const { columns } = useGridColumns(el)
        return { el, columns }
      },
      template: '<div ref="el" style="--grid-columns: 3;"></div>',
    })

    const wrapper = mount(TestComp)
    await wrapper.vm.$nextTick()
    expect(wrapper.vm.columns).toBe(3)
  })
})

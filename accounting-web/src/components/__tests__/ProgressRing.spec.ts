import { mount } from '@vue/test-utils'
import { describe, expect, it } from 'vitest'
import ProgressRing from '../ProgressRing.vue'

function circumferenceOf(size: number): number {
  const strokeWidth = size / 8
  const radius = size / 2 - strokeWidth / 2
  return 2 * Math.PI * radius
}

describe('ProgressRing', () => {
  it('渲染底环与进度弧两个 circle，弧颜色取自 color prop', () => {
    const wrapper = mount(ProgressRing, {
      props: { percentage: 50, color: '#2ecc71' },
    })

    const circles = wrapper.findAll('circle')
    expect(circles).toHaveLength(2)
    expect(circles[1].attributes('stroke')).toBe('#2ecc71')
  })

  it('按百分比计算弧长（stroke-dasharray 圆周长法）', () => {
    const size = 80
    const wrapper = mount(ProgressRing, {
      props: { percentage: 25, color: '#f1c40f', size },
    })

    const c = circumferenceOf(size)
    const arc = wrapper.findAll('circle')[1]
    expect(arc.attributes('stroke-dasharray')).toBe(`${c * 0.25} ${c}`)
  })

  it('percentage 为 0 时不渲染弧（避免 round linecap 产生圆点）', () => {
    const wrapper = mount(ProgressRing, {
      props: { percentage: 0, color: '#2ecc71' },
    })

    expect(wrapper.findAll('circle')).toHaveLength(1)
  })

  it('percentage 超过 100 时弧按 100 绘制', () => {
    const wrapper = mount(ProgressRing, {
      props: { percentage: 137, color: '#e74c3c' },
    })

    const c = circumferenceOf(64)
    const arc = wrapper.findAll('circle')[1]
    expect(arc.attributes('stroke-dasharray')).toBe(`${c} ${c}`)
  })

  it('渲染默认 slot 的中心内容', () => {
    const wrapper = mount(ProgressRing, {
      props: { percentage: 42, color: '#2ecc71' },
      slots: { default: '42%' },
    })

    expect(wrapper.find('.progress-ring-center').text()).toBe('42%')
  })

  it('无 slot 时中心回退显示百分比文本', () => {
    const wrapper = mount(ProgressRing, {
      props: { percentage: 42, color: '#2ecc71' },
    })

    expect(wrapper.find('.progress-ring-center').text()).toBe('42%')
  })

  it('size prop 决定 svg 宽高', () => {
    const wrapper = mount(ProgressRing, {
      props: { percentage: 10, color: '#2ecc71', size: 96 },
    })

    const svg = wrapper.find('svg')
    expect(svg.attributes('width')).toBe('96')
    expect(svg.attributes('height')).toBe('96')
  })
})

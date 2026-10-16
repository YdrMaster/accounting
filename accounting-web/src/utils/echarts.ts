import { LineChart, SunburstChart } from 'echarts/charts'
import { GraphicComponent, GridComponent, TooltipComponent } from 'echarts/components'
import * as echarts from 'echarts/core'
import { CanvasRenderer } from 'echarts/renderers'

echarts.use([
  LineChart,
  SunburstChart,
  CanvasRenderer,
  GraphicComponent,
  GridComponent,
  TooltipComponent,
])

export { echarts }

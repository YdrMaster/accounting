import * as echarts from 'echarts/core'
import { LineChart, SunburstChart } from 'echarts/charts'
import { CanvasRenderer } from 'echarts/renderers'
import { GraphicComponent, GridComponent, TooltipComponent } from 'echarts/components'

echarts.use([
  LineChart,
  SunburstChart,
  CanvasRenderer,
  GraphicComponent,
  GridComponent,
  TooltipComponent,
])

export { echarts }

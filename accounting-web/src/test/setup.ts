import { Window } from 'happy-dom'

// Node 26+ 注入原生实验性 localStorage（需 --localstorage-file 参数才可用），
// 这个不可用的占位会顶掉 happy-dom 注入的实现，导致测试环境拿不到可用的
// localStorage。检测到缺失时，用独立 Window 的存储实现补回。
const storageWindow = new Window()

if (typeof localStorage === 'undefined') {
  Object.defineProperty(globalThis, 'localStorage', {
    value: storageWindow.localStorage,
    configurable: true,
    writable: true,
  })
}

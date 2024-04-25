import './App.css'
import MainPage from './pages/main/MainPage'

import { ChakraProvider } from '@chakra-ui/react'

function App() {
  return (
    <ChakraProvider>
      <MainPage />
    </ChakraProvider>
  )
}

export default App

import { useState } from 'react'
import './App.css'
import { Button } from '@/core/presentation/ui/button'

function App() {
  const [count, setCount] = useState(0)

  return (
    <>
        <Button onClick={ () => setCount(count + 1)}>Increment</Button>
        <p>Count : {count} </p>
    </>
  )
}

export default App

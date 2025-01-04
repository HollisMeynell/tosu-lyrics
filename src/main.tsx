import {StrictMode} from 'react'
import {createRoot} from 'react-dom/client'
import './index.css';
import LyricsBox from './components/lyrics-box'
import CenterBox from "./components/center-box";

createRoot(document.getElementById('root')!).render(
    <StrictMode>
        <CenterBox>
            <LyricsBox/>
        </CenterBox>
    </StrictMode>,
)

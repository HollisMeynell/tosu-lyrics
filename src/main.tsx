/* @refresh reload */

import { render } from 'solid-js/web';
import './index.css';
import LyricsBox from './components/lyrics-box'
import CenterBox from "./components/center-box";

const root = document.getElementById('root');

if (import.meta.env.DEV && !(root instanceof HTMLElement)) {
    throw new Error(
        'Root element not found. Did you forget to add it to your index.html? Or maybe the id attribute got misspelled?',
    );
}

const main = () => <CenterBox>
    <LyricsBox/>
</CenterBox>

render(main, document.getElementById('root')!)

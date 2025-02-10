const fontURL = new URL("/LRC.otf", import.meta.url).href;
const originFontURL = new URL("/oLRC.otf", import.meta.url).href;
const TranslateFontURL = new URL("/tLRC.otf", import.meta.url).href;

export const loadFont = async () => {
    // fixme: 现在字体貌似没有为原歌词跟翻译歌词细分, 这方面你打算怎么处理
    return loadDefaultFont();
};

export const loadDefaultFont = async () => {
    const fontFace = new FontFace("LRC", `url(${fontURL})`);
    const load = await fontFace.load();
    document.fonts.add(load);
    return "LRC";
};

export const loadOriginFont = async () => {
    const fontFace = new FontFace("oLRC", `url(${originFontURL})`);
    const load = await fontFace.load();
    document.fonts.add(load);
    return "oLRC";
};

export const loadTranslateFont = async () => {
    const fontFace = new FontFace("tLRC", `url(${TranslateFontURL})`);
    const load = await fontFace.load();
    document.fonts.add(load);
    return "tLRC";
};

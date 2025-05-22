export default function () {
    return (
        <>
            <form
                action="/api/font/upload"
                method="post"
                enctype="multipart/form-data"
            >
                <input
                    type="file"
                    name="font"
                    accept="font/ttf, font/otf, font/woff, font/woff2"
                />
                <input type="submit" value="upload" />
            </form>
        </>
    );
}

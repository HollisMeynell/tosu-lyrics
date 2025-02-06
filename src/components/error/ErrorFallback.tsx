/**
 * 错误回退组件
 * @param err 错误信息
 */
export default function ErrorFallback(err:Error) {
    console.error(err);
    return (
        <div
            class="bg-gradient-to-br from-[#ff9a9e] to-[#fad0c4] text-white p-10 rounded-lg shadow-md text-center max-w-[500px] mx-auto my-12 font-sans">
            <h2 class="text-2xl mb-5">⚠️ Oops! Something went wrong</h2>
            <div class="bg-white/20 p-4 rounded mb-5">
                <p class="m-0 break-words">{err.toString()}</p>
            </div>
            <button
                class="bg-white text-[#ff6f61] border-none px-5 py-2.5 rounded cursor-pointer text-base font-bold shadow-sm hover:bg-gray-100 transition-colors duration-300"
                onClick={() => window.location.reload()}
            >
                Reload Page
            </button>
            <div class="mt-5">
                <p class="text-sm text-white/80">
                    If the problem persists, please contact support.
                </p>
            </div>
        </div>
    );
}

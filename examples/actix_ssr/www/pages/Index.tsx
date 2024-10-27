import { Head, Link } from "@inertiajs/react"
import { ArrowSquareOut } from "@phosphor-icons/react/dist/ssr/ArrowSquareOut";
import { useState } from "react"

type Props = {
    version: string,
    message: string
}

export default function Index({message, version}: Props) {
    const [count, setCount] = useState(0);
    const increment = () => setCount(prev => ++prev);

    return (
        <>
            <Head>
                <title>Hello, from inertia-rust!</title>
                <meta name="description" content="Just a mocked head... Ha!" />
            </Head>

            <main className="w-full h-full flex flex-col justify-center items-center">
                <h1 className="text-6xl font-black text-center mb-5">Yeah!<br/>Inertia-Rust v{version}</h1>
                <p className="text-xl font-medium text-center mb-12">{message}</p>

                <div className="flex flex-col items-center gap-4 w-fit">
                    <div className="rounded-2xl bg-white/10 h-20 w-full grid place-items-center">
                        <span className="font-black text-4xl">{count}</span>
                    </div>
                    <button
                        className="
                        p-7 py-4 rounded-xl bg-purple-700 hover:bg-purple-800 active:bg-purple-900
                        transition-all duration-100 ring-0 ring-purple-600/25 focus:ring-8 outline-none
                        select-none font-medium text-xl cursor-default
                        "
                        onClick={increment}
                    >
                        Taste this state!
                    </button>

                    <Link
                        href="/contact"
                        className="
                            text-purple-200 font-medium text-lg underline
                            flex items-center gap-3 bg-purple-400/25 px-8 py-3 rounded-xl
                        "
                    >
                        Other page? <ArrowSquareOut size={24} weight="bold" />
                    </Link>
                </div>
            </main>
        </>
    )
}

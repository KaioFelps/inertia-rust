import { Head, Link } from "@inertiajs/react";
import { ArrowLeft } from "@phosphor-icons/react/dist/ssr";
import { useEffect } from "react";

export default function Index(props: any) {
    useEffect(() => {console.log(props)}, [])
    return (
        <>
            <Head title="Foo Home Page!"></Head>

            <main className="w-full h-full flex flex-col justify-center items-center">
                <h1 className="text-6xl font-black text-center mb-5">Simplest page ever seen!</h1>
                <p className="text-xl font-medium text-center mb-12">
                    Nothing much to see here, indeed...
                </p>

                <div className="flex flex-col items-center gap-4 w-fit">
                    <Link
                        href="/"
                        className="
                            text-purple-200 font-medium text-lg underline
                            flex items-center gap-3 bg-purple-400/25 px-8 py-3 rounded-xl
                        "
                    >
                        <ArrowLeft size={24} weight="bold" />
                        Go back, then?
                    </Link>
                </div>
            </main>
        </>
    )
}
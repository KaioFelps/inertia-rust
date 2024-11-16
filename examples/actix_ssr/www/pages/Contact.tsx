import { Head, Link } from "@inertiajs/react"
import { ArrowLeft } from "@phosphor-icons/react/dist/ssr/ArrowLeft"

type Props = {
    user: {
        name: string,
        email: string,
    }
    assetsVersion: string,
}

export default function Contact({ user, assetsVersion }: Props) {
    return (
        <>
            <Head title={"My name is " + user.name + "!"} />
            <main className="h-full flex flex-col items-center justify-center">
                <h1 className="text-6xl font-black mb-6">Hey! I'm {user.name}</h1>
                <p className="text-lg mb-12">Contact-me: <span className="italic">{user.email}</span></p>

                <Link
                    href="/"
                    className="
                        group text-purple-200 font-medium text-lg underline
                        flex items-center gap-3 bg-purple-400/25 hover:bg-purple-400/35
                        px-8 py-3 rounded-xl cursor-default
                    "
                >
                    <span className="transition-all relative left-0 group-hover:-left-1"><ArrowLeft size={24} weight="bold" /></span>
                    Back to home!
                </Link>

                <span className="flex w-fit px-5 py-2 rounded-lg bg-purple-50/10 mt-8 text-purple-50">
                    Assets version: {assetsVersion}
                </span>
            </main>
        </>
    )
}
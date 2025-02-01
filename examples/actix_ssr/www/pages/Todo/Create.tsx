import { Head, useForm } from "@inertiajs/react";
import { FormEvent } from "react";

type FormProps = {
    title: string;
    content: string;
}

export default function CreateTask() {
    const {setData, post, errors} = useForm<FormProps>();
    function handleSubmit(e: FormEvent<HTMLFormElement>) {
        e.preventDefault();
        post("/todo/store");
    }

    return (
    <>
        <Head>
            <title>create a new to-do task</title>
        </Head>
        <main className="w-full max-w-[calc(100%_-_96px)] mx-auto h-full flex flex-col justify-center items-center">
            <form
                onSubmit={handleSubmit}
                className="flex flex-col p-4 rounded-lg bg-white/5 border border-white/15 w-96"
            >
                <h1 className="font-bold mb-3 text-white/70">Create a new task</h1>

                <label className="mb-4">
                    <span className="block mb-2 text-sm">Title</span>
                    {errors.title && (
                        <span
                            className="block mb-1 text-red-500 bg-red-400/10 px-1 rounded-md text-sm"
                        >
                            {errors.title}
                        </span>
                    )}
                    <input
                        type="text"
                        placeholder="I gotta do..."
                        onInput={(e) => setData((data) => ({...data, title: (e.target as HTMLInputElement).value}))}
                        className="
                            bg-white/10 rounded-md px-2 py-1 outline-none ring-0 ring-purple-400 focus:ring-2
                            text-sm w-full
                        "
                    />
                </label>

                <label>
                    <span className="block mb-2 text-sm">Content</span>
                    {errors.content && (
                        <span
                            className="block mb-1 text-red-500 bg-red-400/10 px-1 rounded-md text-sm"
                        >
                            {errors.content}
                        </span>
                    )}
                    <input
                        type="text"
                        placeholder="I gotta do..."
                        onInput={(e) => setData((data) => ({...data, content: (e.target as HTMLInputElement).value}))}
                        className="
                            bg-white/10 rounded-md px-2 py-1 outline-none ring-0 ring-purple-400 focus:ring-2
                            text-sm w-full
                        "
                    />
                </label>

                <button
                    className="
                        mt-3 self-start
                        px-3 py-1 rounded-md bg-purple-700 hover:bg-purple-800 active:bg-purple-900
                        transition-all duration-100 ring-0 ring-purple-600/25 focus:ring-4 outline-none
                        select-none font-medium text-sm cursor-default
                    "
                >
                    Create
                </button>
            </form>
        </main>
    </>
    )
}
import { Head, useForm } from "@inertiajs/react";
import { FormEvent } from "react";

type FormProps = {
    content: string;
}

export default function CreateTask() {
    const {setData, post, wasSuccessful, errors} = useForm<FormProps>();
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
                className="flex flex-col p-4 rounded-lg bg-white/5 border border-white/15"
            >
                <h1 className="font-bold mb-3 text-white/70">Create a new task</h1>

                {wasSuccessful && (
                    <span className="text-green-500 bg-green-400/30 px-2 py-1 mb-3 -mx-1 text-sm rounded-md">
                        Successfully created!
                    </span>
                )}

                <label>
                    <span className="block mb-2 text-sm">Content</span>
                    {errors.content && (
                        <span
                            className="text-red-500 bg-red-400/30 px-1 rounded-md"
                        >
                            {errors.content}
                        </span>
                    )}
                    <input
                        type="text"
                        placeholder="I gotta do..."
                        onInput={(e) => setData({content: (e.target as HTMLInputElement).value})}
                        className="
                            bg-white/10 rounded-md px-2 py-1 outline-none ring-0 ring-purple-400 focus:ring-2
                            text-sm
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
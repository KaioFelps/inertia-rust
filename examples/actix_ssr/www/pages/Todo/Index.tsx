import { Deferred, Head, router, usePage } from "@inertiajs/react";
import { useCallback } from "react";

type Task = {
    title: string;
    description: string;
    done: boolean;
}

type TodoProps = {
    page: number,
    tasks: Task[]
}

export default function Todo() {
    const page = usePage<TodoProps>().props.page;
    
    const handleLoadMoreTasks = useCallback(() => {
        router.reload({
            data: { page: page + 1 },
            only: ["page", "tasks"],
        })
    }, [page]);

    const handleLoadOnlyCurrentTasks = useCallback(() => {
        router.reload({
            data: { page },
            only: ["tasks"],
            reset: [ "tasks" ]
        })
    }, [page]);

    return (
        <>
            <Head>
                <title>inertia-rust to-do!</title>
                <meta name="description" content="The dumbiest to-do list you've ever seen!" />
            </Head>
            <main className="w-full max-w-[calc(100%_-_96px)] mx-auto h-full flex flex-col justify-center items-center">
                <h1 className="text-6xl font-black text-center mb-5">To-do Tasks</h1>

                <Deferred fallback="Loading tasks..." data="tasks"  >
                    <TasksList />
                </Deferred>

                <div className="my-6 flex gap-2 items-center justify-center">
                    <button
                        className="
                            p-7 py-4 rounded-xl bg-purple-700 hover:bg-purple-800 active:bg-purple-900
                            transition-all duration-100 ring-0 ring-purple-600/25 focus:ring-8 outline-none
                            select-none font-medium text-xl cursor-default
                        "
                        onClick={handleLoadMoreTasks}
                    >
                        Load few more!
                    </button>

                    <button
                        className="
                            p-7 py-4 rounded-xl bg-purple-500/20 hover:bg-purple-600/20 active:bg-purple-800/20
                            transition-all duration-100 ring-0 ring-purple-400/25 focus:ring-8 outline-none
                            select-none font-medium text-xl cursor-default
                        "
                        onClick={handleLoadOnlyCurrentTasks}
                    >
                        Show only the current page! 😡
                    </button>
                </div>
            </main>
        </>
    )
}

function TasksList() {
    const { tasks } = usePage<TodoProps>().props;

    return (
        <div className="flex flex-col gap-3">
            {(tasks).map(task => (
                <div
                    data-active={task.done} key={task.title}
                    className="
                        rounded-2xl bg-white/10 w-full flex flex-col gap-1 p-5
                    "
                >
                    <h2 className="my-0 text-lg font-bold">{task.title}</h2>
                    <p className="text-white/80 leading-normal">{task.description}</p>
                </div>
            ))}
        </div>
    )
}
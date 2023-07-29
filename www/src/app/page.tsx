'use client';

export default function Home() {
  const submitHandler = async () => {
    const rregex = (await import('rregex'));
    const post = rregex.get_debug_graph_json("a|b*");
    console.log(post.get_dot_str());
    console.log(post.get_start());
    console.log(post.get_fin());
    post.free();
    rregex.greet();
  };

  return (
    <main>
      <h1 className="text-3xl">RRegex</h1>
      <button
        onClick={() => {
          (async () => {
            await submitHandler();
          })().catch((e: unknown) => {
            console.error(`Could not submit: ${e}`);
          });
        }}
      >
        Click here
      </button>
    </main>
  );
}

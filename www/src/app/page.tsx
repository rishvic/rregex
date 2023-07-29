'use client';

export default function Home() {
  const submitHandler = async () => {
    const rregex = (await import('rregex'));
    const post = rregex.get_debug_postexpr_string("ab|(b*f|e)*cd");
    console.log(post);
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

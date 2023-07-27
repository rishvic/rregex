'use client';

export default function Home() {
  const submitHandler = async () => {
    (await import('rregex')).greet();
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

'use client';
import {Graphviz} from 'graphviz-react';
import {useState} from 'react';

export default function Home() {

  const [dotStr, setDotStr] = useState<string>("");

  const submitHandler = async () => {
    const rregex = (await import('rregex'));
    const post = rregex.get_debug_graph_json("a|bc*");
    setDotStr(post.get_dot_str());
    console.log(post.get_dot_str());
    console.log(post.get_start());
    console.log(post.get_fin());
    post.free();
    rregex.greet();
  };

  return (
    <main>
      <h1 className="text-3xl">RRegex</h1>
      <p>
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
      </p>
      {dotStr && <Graphviz dot={dotStr} />}
    </main>
  );
}

import Link from 'next/link';

export default function Home() {
  return (
    <main className="container mx-auto mt-8">
      <h1 className="text-2xl">List of assignments</h1>
      <ul className="list-disc">
        <li>
          <Link
            className="font-medium text-blue-600 dark:text-blue-500 hover:underline"
            href="/rregex"
          >
            Rregex
          </Link>
        </li>
      </ul>
    </main>
  );
}

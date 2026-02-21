function Channel({ text }: { text: string }) {
  return (
    <div className="flex flex-col gap-4 text-stone-300 text-l font-sans-serif ml-4 p-1.5 rounded-xl hover:bg-stone-900">
      <p>{text}</p>
    </div>
  );
}

export default Channel;
import {
  Card,
  CardBody,
  CardFooter,
  CardHeader,
  Link,
} from "@nextui-org/react";

import ossFriends from "@/oss-friends.json";

export default function OssFriendsPage() {
  const data = ossFriends.data;

  return (
    <section id="oss-friends">
      <div className="relative overflow-hidden mt-24">
        <main className="max-w-8xl relative mx-auto mb-auto flex w-full flex-col justify-center sm:px-2 lg:px-8 xl:px-12">
          <div className="px-4 py-20 text-center sm:px-6 lg:px-8 lg:py-28">
            <h1 className="text-3xl font-bold tracking-tight text-black-100 sm:text-4xl md:text-5xl">
              <span className="xl:inline">Our</span>{" "}
              <span className="text-indigo-600 xl:inline">Open-source </span>
              <span className="inline ">Friends</span>
            </h1>
            <p className="mx-auto mt-3 max-w-md text-base text-slate-200 dark:text-slate-300 sm:text-lg md:mt-5 md:max-w-2xl md:text-xl">
              In Hook0, we are proud to collaborate with a diverse group of
              partners to promote open-source software and the values of
              transparency, collaboration, and community that it represents.
            </p>
          </div>
          <div className="pt-10 sm:pt-16 lg:pt-8 lg:pb-0 -mt-16">
            <div className="mx-auto max-w-7xl lg:px-8">
              <div className="m-4 grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
                {data.map((item, idx) => (
                  <Card key={idx} className="max-w-[400px]">
                    <CardHeader className="flex gap-3">
                      <div className="flex flex-col">
                        <Link
                          isExternal
                          className="text-lg font-semibold text-white"
                          href={item.href}
                        >
                          {item.name}
                        </Link>
                      </div>
                    </CardHeader>
                    <CardBody>
                      <p>{item.description}</p>
                    </CardBody>
                    <CardFooter>
                      <Link
                        isExternal
                        showAnchorIcon
                        className="text-indigo-400"
                        href="https://github.com/nextui-org/nextui"
                      >
                        Learn more
                      </Link>
                    </CardFooter>
                  </Card>
                ))}
              </div>
            </div>
          </div>
        </main>
      </div>
    </section>
  );
}

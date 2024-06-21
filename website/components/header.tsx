"use client";
import React from "react";
import Image from "next/image";

import { ContainerScroll } from "./ui/container-scroll-animation";
import { GetStartedForFreeButton } from "./ui/hover-border-gradient";

export function Header() {
  return (
    <div className="flex flex-col overflow-hidden -mt-2 md:-mt-12">
      <ContainerScroll
        titleComponent={
          <>
            <h1 className="text-4xl font-semibold text-black dark:text-white">
              Webhooks <span className="text-indigo-400">As A Service</span>
              <br />
              <span className="text-4xl md:text-[6rem] font-bold mt-1 leading-none">
                Hook0
              </span>
            </h1>
          </>
        }
      >
        <Image
          alt="hero"
          className="mx-auto rounded-2xl object-cover h-full object-left-top hidden md:block"
          draggable={false}
          height={720}
          src={`/hook0-app.png`}
          width={1400}
        />
        <Image
          alt="hero"
          className="mx-auto rounded-2xl object-cover h-full object-left-top md:hidden"
          draggable={false}
          height={720}
          src={`/hook0-app-mobile.png`}
          width={1400}
        />
      </ContainerScroll>
      <div className="-mt-36">
        <h2 className="text-2xl text-center">
          <span className="text-indigo-400">Open-Source</span>,{" "}
          <span className="text-indigo-400">Free trial</span>,{" "}
          <span className="text-indigo-400">No credit card required</span>,{" "}
          <span className="text-indigo-400">Cancel Anytime</span>
        </h2>
        <p className="mt-8 text-center text-lg">
          Hook0 is an Open-Source Webhooks-as-a-service (WaaS) that makes it
          easy for developers to send webhooks. Developers make one API call,
          and Hook0 takes care of deliverability, retries, security, and more.
        </p>
        <GetStartedForFreeButton />
      </div>
    </div>
  );
}

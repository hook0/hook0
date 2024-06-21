import Image from "next/image";

export function Testimonials() {
  return (
    <section className="mt-36" id="testimonials">
      <h2 className="text-4xl font-bold text-center mb-10">Testimonials</h2>
      <div className="rounded bg-indigo-800 max-w-7xl mx-auto md:grid md:grid-cols-2 md:px-6 lg:px-8">
        <div className="py-12 px-4 sm:px-6 md:flex md:flex-col md:py-16 md:pl-0 md:pr-10 md:border-r md:border-indigo-900 lg:pr-16">
          <blockquote className="mt-6 md:flex-grow md:flex md:flex-col">
            <div className="relative text-lg font-medium text-white md:flex-grow">
              <svg
                aria-hidden="true"
                className="absolute top-0 left-0 -translate-x-3 -translate-y-2 h-8 w-8 text-indigo-600"
                fill="currentColor"
                viewBox="0 0 32 32"
              >
                <path d="M9.352 4C4.456 7.456 1 13.12 1 19.36c0 5.088 3.072 8.064 6.624 8.064 3.36 0 5.856-2.688 5.856-5.856 0-3.168-2.208-5.472-5.088-5.472-.576 0-1.344.096-1.536.192.48-3.264 3.552-7.104 6.624-9.024L9.352 4zm16.512 0c-4.8 3.456-8.256 9.12-8.256 15.36 0 5.088 3.072 8.064 6.624 8.064 3.264 0 5.856-2.688 5.856-5.856 0-3.168-2.304-5.472-5.184-5.472-.576 0-1.248.096-1.44.192.48-3.264 3.456-7.104 6.528-9.024L25.864 4z" />
              </svg>
              <p className="relative">
                Give it enough time and open-Source always wins in the end!
              </p>
            </div>
            <footer className="mt-8">
              <div className="flex items-start">
                <div className="flex-shrink-0 inline-flex rounded-full border-2 border-white">
                  <Image
                    alt="François-Guillaume Ribreau"
                    className="h-12 w-12 rounded-full"
                    height={400}
                    src="/team/francois-guillaume-ribreau.jpeg"
                    width={400}
                  />
                </div>
                <div className="ml-4">
                  <div className="text-base font-medium text-white">
                    François-Guillaume Ribreau
                  </div>
                  <div className="text-base font-medium text-indigo-200">
                    Hook0 co-founder
                  </div>
                </div>
              </div>
            </footer>
          </blockquote>
        </div>
        <div className="py-12 px-4 border-t-2 border-indigo-900 sm:px-6 md:py-16 md:pr-0 md:pl-10 md:border-t-0 md:border-l lg:pl-16">
          <blockquote className="mt-6 md:flex-grow md:flex md:flex-col">
            <div className="relative text-lg font-medium text-white md:flex-grow">
              <svg
                className="absolute top-0 left-0 -translate-x-3 -translate-y-2 h-8 w-8 text-indigo-600"
                fill="currentColor"
                viewBox="0 0 32 32"
              >
                <path d="M9.352 4C4.456 7.456 1 13.12 1 19.36c0 5.088 3.072 8.064 6.624 8.064 3.36 0 5.856-2.688 5.856-5.856 0-3.168-2.208-5.472-5.088-5.472-.576 0-1.344.096-1.536.192.48-3.264 3.552-7.104 6.624-9.024L9.352 4zm16.512 0c-4.8 3.456-8.256 9.12-8.256 15.36 0 5.088 3.072 8.064 6.624 8.064 3.264 0 5.856-2.688 5.856-5.856 0-3.168-2.304-5.472-5.184-5.472-.576 0-1.248.096-1.44.192.48-3.264 3.456-7.104 6.528-9.024L25.864 4z" />
              </svg>
              <p className="relative">
                We are building it the way we would love to use it
              </p>
            </div>
            <footer className="mt-8">
              <div className="flex items-start">
                <div className="flex-shrink-0 inline-flex rounded-full border-2 border-white">
                  <Image
                    alt="David Sferruzza"
                    className="h-12 w-12 rounded-full"
                    height={400}
                    src="/team/david-sferruzza.jpeg"
                    width={400}
                  />
                </div>
                <div className="ml-4">
                  <div className="text-base font-medium text-white">
                    David Sferruzza, PhD
                  </div>
                  <div className="text-base font-medium text-indigo-200">
                    Hook0 co-founder
                  </div>
                </div>
              </div>
            </footer>
          </blockquote>
        </div>
      </div>
    </section>
  );
}

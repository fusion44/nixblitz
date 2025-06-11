pub(crate) mod buttons {
    pub(crate) const DEFAULT: &str = "inline-flex items-center justify-center border align-middle select-none font-sans font-medium text-center transition-all duration-300 ease-in disabled:opacity-50 disabled:shadow-none disabled:cursor-not-allowed data-[shape=pill]:rounded-full data-[width=full]:w-full focus:shadow-none text-sm rounded-lg py-2.5 px-6 shadow-md hover:shadow-lg bg-slate-700 border-slate-600 text-slate-50 hover:bg-slate-600 hover:border-slate-500 focus:ring-4 focus:ring-slate-500/50";
    pub(crate) const GHOST: &str = "inline-flex border font-medium font-sans text-center transition-all duration-300 ease-in disabled:opacity-50 disabled:shadow-none disabled:cursor-not-allowed data-[shape=pill]:rounded-full data-[width=full]:w-full focus:shadow-none text-sm rounded-md py-2 px-4 bg-transparent border-transparent text-slate-800 hover:bg-slate-800/5 hover:border-slate-800/5 shadow-none hover:shadow-none";
    pub(crate) const OUTLINE: &str = "inline-flex border font-medium font-sans text-center transition-all duration-300 ease-in disabled:opacity-50 disabled:shadow-none disabled:cursor-not-allowed data-[shape=pill]:rounded-full data-[width=full]:w-full focus:shadow-none text-sm rounded-md py-2 px-4 shadow-sm hover:shadow-md bg-transparent border-slate-800 text-slate-800 hover:bg-slate-800 hover:text-slate-50";
    pub(crate) const SOLID: &str = "inline-flex border font-medium font-sans text-center transition-all duration-300 ease-in disabled:opacity-50 disabled:shadow-none disabled:cursor-not-allowed data-[shape=pill]:rounded-full data-[width=full]:w-full focus:shadow-none text-sm rounded-md py-2 px-4 shadow-sm hover:shadow-md bg-slate-800 border-slate-800 text-slate-50 hover:bg-slate-700 hover:border-slate-700";
    pub(crate) const GRADIENT: &str = "inline-flex border font-medium font-sans text-center transition-all duration-300 ease-in disabled:opacity-50 disabled:shadow-none disabled:cursor-not-allowed data-[shape=pill]:rounded-full data-[width=full]:w-full focus:shadow-none text-sm rounded-md py-2 px-4 shadow-sm hover:shadow-md bg-gradient-to-tr from-slate-800 to-slate-700 border-slate-800 text-slate-50 hover:brightness-105";
}
pub(crate) mod typography {
    pub(crate) mod headings {
        pub(crate) const H1: &str =
            "font-sans text-4xl font-bold antialiased md:text-5xl lg:text-6xl";
        pub(crate) const H2: &str =
            "font-sans text-3xl font-bold antialiased md:text-4xl lg:text-5xl";
        pub(crate) const H3: &str =
            "font-sans text-2xl font-bold antialiased md:text-3xl lg:text-4xl";
        pub(crate) const H4: &str =
            "font-sans text-xl font-bold antialiased md:text-2xl lg:text-3xl";
        pub(crate) const H5: &str =
            "font-sans text-lg font-bold antialiased md:text-xl lg:text-2xl";
        pub(crate) const H6: &str =
            "font-sans text-base font-bold antialiased md:text-lg lg:text-xl";
    }
}

pub mod trend {
    use async_trait::async_trait;
    use derive_more::Constructor;
    use gen_span::gen_span;
    use sqlx::SqlitePool;
    use uuid::fmt::Simple;
    use crate::backend::{
        GetTrendError, IDOnlyEntity, ServiceError, TrendEntry, TrendService,
        TrendingHashtag,
    };
    pub struct DBTrendService {
        pool: SqlitePool,
    }
    #[allow(missing_docs)]
    #[automatically_derived]
    impl DBTrendService {
        #[inline]
        pub const fn new(pool: SqlitePool) -> DBTrendService {
            DBTrendService { pool: pool }
        }
    }
    struct HashtagCount {
        hashtag: String,
        count: i64,
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for HashtagCount {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field2_finish(
                f,
                "HashtagCount",
                "hashtag",
                &self.hashtag,
                "count",
                &&self.count,
            )
        }
    }
}

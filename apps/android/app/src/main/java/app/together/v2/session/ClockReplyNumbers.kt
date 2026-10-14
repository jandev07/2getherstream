package app.together.v2.session

/** The PC sends integer milliseconds. Reject JSON coercions and integers which
 * would lose precision when passed to the double-based clock estimator. */
internal fun clockReplyMillis(value:Any?):Double? = MediaGrantNumbers.integer(value)
    ?.takeIf {it in 0L..9007199254740991L}?.toDouble()
